# noodles

The only command line client worthy of pglbutts

## How

```bash
git clone https://github.com/pglbutt/noodles.git
cd noodles
virtualenv env
source env/bin/activate
pip install -r requirements.txt
pip install --editable .
spag
Hello World!
```

## Future Features

1. Set an __endpoint__ that lets you do subsequent shorter requests to that endpoint.
2. Bring __collections__ or something similar that lets you do short/simple commands `spag create_foo`
3. Be able to grab URI components or response body attributes from previous requests. ie `spag /v2/zones/!id`
4. Be able to generate a log of requests that have been dispatched `spag log`
