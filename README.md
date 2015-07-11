# noodles

[![Build Status](https://travis-ci.org/pglbutt/noodles.svg?branch=master)](https://travis-ci.org/pglbutt/noodles)

Noodles is a collection of command line tools designed to make your life better.

# spag

spag is a __command line REST client__. It aims to make developing, testing, and
using REST APIs as easy as possible. spag lets you:

- Switch between multiple environmenst with variables for use with multiple APIs.
- Define parameterized requests in YAML files.
- Grab data from the previous response for use in future requests.
- Examine a detailed history of requests.


## Building/running/testing

The project is built as a lib by default, which lets us import the crate whenever
we need to test something:

    cargo test

To build and run `spag`, just do:

    cargo run

Or also:

    cargo run --bin spag

## Basics

  - Spag can replace curl for most requests. It provides a wealth of other
    features to make it more user-friendly.
  - Spag separates the base api endpoint from resource paths to make it easy to
    work with different deployments of the same api.

```bash
$ spag post /things --data '{ "id":"pglbutt" }' -e http://localhost:5000 -H Content-Type:application/json
{
  "id": "pglbutt"
}

$ spag get /things -e http://localhost:5000
{
  "things": [
    {
      "id": "pglbutt"
    }
  ]
}

$ spag put /things/pglbutt --data '{ "id":"pglbutt2" }' -e http://localhost:5000 -H Content-Type:application/json
{
  "id": "pglbutt2"
}

$ spag delete /things/pglbutt2 -e http://localhost:5000

$ spag get /things -e http://localhost:5000
{
  "things": []
}
```

## Environments

Environments allow you to set arbitrary variables to be used in your request.
There are special ones that you probably want to set:

* __endpoint__ - The HTTP endpoint that will be postfixed with your request resource.
* __headers__ - HTTP headers to be used in all your requests
* __dir__ - The directory your templates are in


The default environment is always there.

```bash
$ spag env set endpoint http://localhost:5000 id pglbutt headers.Content-Type application/json
---
"endpoint": "http://localhost:5000"
"headers":
  "Content-Type": "application/json"
"id": "pglbutt"
```

Spag applies your environment variables.

```bash
$ spag post /things --data '{ "id":"pglbutt" }'
{
  "id": "pglbutt"
}

$ spag get /things/pglbutt
{
  "id": "pglbutt"
}
```

You can also create your own environments out of band of spag.

```bash
cat << EOF > .spag/environments/pglbutt.yml
---
"endpoint": "http://localhost:5000"
"headers":
  "Content-Type": "application/json"
  "Some-Other-header": "text"
  "pglbutt": "pglbutt"
EOF

$ spag env activate pglbutt
---
"endpoint": "http://localhost:5000"
"headers":
  "Content-Type": "application/json"
  "Some-Other-header": "text"
  "pglbutt": "pglbutt"


$ spag env show
---
"endpoint": "http://localhost:5000"
"headers":
  "Content-Type": "application/json"
  "Some-Other-header": "text"
  "pglbutt": "pglbutt"

```

## Use Previous Request Data
```bash
$ spag post /things --data '{ "id":"pglbutt" }'
{
  "id": "pglbutt"
}
```

Use `@` to get items from the last response body.

```bash
$ spag get /things/@id
{
  "id": "pglbutt"
}
```

Use `@[].` to get items from the active environment.

```bash
$ spag env set id @id
$ spag get /things/@[].id
{
  "id": "pglbutt"
}
```

## Predefined, Parameterized Requests

Basic templates are just predefined requests, not really templates.

```bash
$ cat templates/post_thing.yml
method: POST
uri: /things
headers:
    Content-Type: "application/json"
    Accept: "application/json"
body: |
    {
        "id": "pglbutt"
    }

$ spag request post_thing
{
  "id": "pglbutt"
}
```

Double braces signify a variable to be subsituted.

```bash
$ cat templates/post_thing.yml
method: POST
uri: /things
headers:
    Content-Type: "application/json"
    Accept: "application/json"
body: |
    {
        "id": "{{thing_id}}"
    }

$ spag request post_thing --with thing_id thing
{
  "id": "thing"
}
```

You can also specify default values.

```bash
$ cat templates/post_thing.yml
method: POST
uri: /things
headers:
    Content-Type: "application/json"
    Accept: "application/json"
body: |
    {
        "id": "{{thing_id: poo}}"
    }

$ spag request post_thing
{
  "id": "poo"
}
```

You can see more examples of request files at
[pglbutt/designate-noodles](https://github.com/pglbutt/designate-noodles),
which is a set of request files for the [OpenStack Designate](http://docs.openstack.org/developer/designate/) project.

## History

```bash
$ spag history
0: GET http://localhost:5000/things/pglbutt
1: POST http://localhost:5000/things
2: POST http://localhost:5000/clear
3: DELETE http://localhost:5000/things/pglbutt
4: GET http://localhost:5000/things/pglbutt
5: POST http://localhost:5000/things
6: GET http://localhost:5000/things
7: POST http://localhost:5000/clear

$ spag history show 1
-------------------- Request ---------------------
POST http://localhost:5000/things
Accept: application/json
Content-Type: application/json
Some-Other-header: text
pglbutt: pglbutt
Body:
{    "id": "thing"}
-------------------- Response ---------------------
Status code 201
content-length: 19
content-type: application/json
date: Sat, 11 Jul 2015 22:33:53 GMT
server: Werkzeug/0.10.4 Python/2.7.5
Body:
{
  "id": "thing"
}
```

## Tests

It's really easy to run the tests.
```bash
pip install nose
pip install -r tests/test-requirements.txt
python tests/testapp.py &
nosetests -sv tests/
```

## Contribute

```bash
git clone https://github.com/pglbutt/noodles.git
cd noodles
virtualenv env
cargo run
git checkout -b new-feature
....
git commit
git push -u fork new-feature
# pull request
```
