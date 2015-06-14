import unittest
import subprocess
import os
import shutil
import json
import textwrap

import yaml

# from spag import remembers
# from spag import files

# TODO: read this from a config?
SPAG_PROG = 'spag'
ENDPOINT = 'http://localhost:5000'
RESOURCES_DIR = os.path.join(os.path.dirname(__file__), 'resources')
TEMPLATES_DIR = os.path.join(os.path.dirname(__file__), 'templates')
V1_RESOURCES_DIR = os.path.join(RESOURCES_DIR, 'v1')
V2_RESOURCES_DIR = os.path.join(RESOURCES_DIR, 'v2')
# SPAG_REMEMBERS_DIR = remembers.SpagRemembers.DIR
# SPAG_HISTORY_FILE = remembers.SpagHistory.FILENAME

def run_spag(*args):
    """
    :returns: A tuple (out, err, ret) where
        out is the output on stdout
        err is the output on stderr
        ret is the exit code
    """
    cmd = [SPAG_PROG] + list(args)
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = p.communicate()
    return (out.decode('utf-8'), err.decode('utf-8'), p.returncode)


class BaseTest(unittest.TestCase):

    # enable long diffs
    maxDiff = None

    @classmethod
    def _rm_remembers_dir(cls):
        try:
            # both os.removedirs and os.rmdir don't work on non-empty dirs
            # shutil.rmtree(SPAG_REMEMBERS_DIR)
            pass
        except OSError:
            pass

    @classmethod
    def _rm_history_file(cls):
        try:
            # os.remove(SPAG_HISTORY_FILE)
            pass
        except OSError:
            pass

    def setUp(self):
        super(BaseTest, self).setUp()
        run_spag('get', '/clear', '-e', ENDPOINT)
        run_spag('env', 'unset', '--everything')
        self._rm_remembers_dir()
        self._rm_history_file()

    def tearDown(self):
        self._rm_remembers_dir()
        self._rm_history_file()
        super(BaseTest, self).tearDown()


class TestHeaders(BaseTest):

    def test_get_no_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {})

    def test_get_one_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H', 'pglbutt:pglbutt')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt"})

    def test_get_two_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT,
                                 '-H', 'pglbutt:pglbutt', '-H', 'wow:wow')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt", "Wow": "wow"})

    def test_get_no_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H')
        self.assertEqual(err, '')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: -H option requires an argument\n')

    def test_get_invalid_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H', 'poo')
        self.assertEqual(err, '')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: Invalid header!\n')

    def test_show_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-h')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertIn('content-type: application/json', out)

    def test_passed_headers_override_environment(self):
        out, err, ret = run_spag('env', 'set', '-H', 'a: b')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(yaml.load(out)['headers'].get('a'), 'b')

        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H', 'a: c')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out).get('A'), 'c')

class TestGet(BaseTest):

    def test_get_no_endpoint(self):
        out, err, ret = run_spag('get', '/auth')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Endpoint not set\n\n')

    def test_get_supply_endpoint(self):
        out, err, ret = run_spag('get', '/auth', '-e', ENDPOINT)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"token": "abcde"})

    def test_get_presupply_endpoint(self):
        out, err, ret = run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        self.assertEqual(out, 'endpoint: {0}\n\n'.format(ENDPOINT))
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        out, err, ret = run_spag('get', '/things')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"things": []})


class TestPost(BaseTest):

    def test_spag_post(self):
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')

class TestPut(BaseTest):

    def test_spag_put(self):
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('put', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "b"})
        self.assertEquals(err, '')

class TestPatch(BaseTest):

    def test_spag_patch(self):
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('patch', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "b"})
        self.assertEquals(err, '')

class TestDelete(BaseTest):

    def test_spag_delete(self):
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('delete', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type:application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(out, '\n')
        self.assertEquals(err, '')
        out, err, ret = run_spag('get', '/things')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"things": []})


@unittest.skip("not implemented")
class TestSpagFiles(BaseTest):

    def setUp(self):
        super(TestSpagFiles, self).setUp()
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        run_spag('env', 'set', 'dir=%s' % RESOURCES_DIR)
        self.table = files.SpagFilesLookup(RESOURCES_DIR)

    def test_spag_lookup(self):
        expected = {
            'auth.yml': set([
                os.path.join(RESOURCES_DIR, 'auth.yml')]),
            'delete_thing.yml': set([
                os.path.join(RESOURCES_DIR, 'delete_thing.yml')]),
            'patch_thing.yml': set([
                os.path.join(V2_RESOURCES_DIR, 'patch_thing.yml')]),
            'post_thing.yml': set([
                os.path.join(V1_RESOURCES_DIR, 'post_thing.yml'),
                os.path.join(V2_RESOURCES_DIR, 'post_thing.yml')]),
            'get_thing.yml': set([
                os.path.join(V1_RESOURCES_DIR, 'get_thing.yml'),
                os.path.join(V2_RESOURCES_DIR, 'get_thing.yml')]),
            'headers.yml': set([
                os.path.join(RESOURCES_DIR, 'headers.yml')]),
        }
        self.assertEqual(self.table, expected)

    def test_spag_load_file(self):
        content = files.load_file(os.path.join(RESOURCES_DIR, 'auth.yml'))
        self.assertEqual(content['method'], 'GET')
        self.assertEqual(content['uri'], '/auth')
        self.assertEqual(content['headers'], {'Accept': 'application/json'})

    def test_spag_request_get(self):
        for name in ('auth.yml', 'auth'):
            out, err, ret = run_spag('request', name)
            self.assertEqual(ret, 0)
            self.assertEqual(json.loads(out), {"token": "abcde"})
            self.assertEqual(err, '')

    def test_spag_request_post(self):
        out, err, ret = run_spag('request', 'v2/post_thing.yml')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"id": "c"})
        self.assertEqual(err, '')

    def test_spag_request_patch(self):
        # stuff in patch_thing.yml needs to match stuff here
        _, _, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                              '-H', 'content-type:application/json')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('request', 'patch_thing.yml')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"id": "c"})
        self.assertEqual(err, '')

    def test_spag_request_delete(self):
        for name in ('delete_thing.yml', 'delete_thing'):
            _, _, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                  '-H', 'content-type:application/json')
            self.assertEqual(ret, 0)
            out, err, ret = run_spag('request', name)
            self.assertEqual(ret, 0)
            self.assertEqual(out, '\n')
            self.assertEqual(err, '')

    def test_spag_request_data_option_overrides(self):
        out, err, ret = run_spag('request', 'v2/post_thing.yml',
                                 '--data', '{"id": "xyz"}',
                                 '-H', 'content-type:application/json')
        self.assertEqual(json.loads(out), {"id": "xyz"})
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

    def test_spag_request_headers_override(self):
        out, err, ret = run_spag('request', 'headers.yml',
                                 '-H', 'Hello:abcde')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"Hello": "abcde"})
        self.assertEqual(ret, 0)

    def test_spag_show_requests(self):
        out, err, ret = run_spag('request', '--show')
        def parse(text):
            return text.split()
        expected = """
            tests/resources/auth.yml
            tests/resources/delete_thing.yml
            tests/resources/headers.yml
            tests/resources/v1/get_thing.yml
            tests/resources/v1/post_thing.yml
            tests/resources/v2/get_thing.yml
            tests/resources/v2/patch_thing.yml
            tests/resources/v2/post_thing.yml
            """
        self.assertEqual(err, '')
        self.assertEqual(parse(out), parse(expected))
        self.assertEqual(ret, 0)

    def test_spag_show_single_request(self):
        out, err, ret = run_spag('request', 'auth.yml', '--show')
        self.assertEqual(err, '')
        self.assertEqual(out.strip(),
            textwrap.dedent("""
            File tests/resources/auth.yml
            method: GET
            uri: /auth
            headers:
                Accept: "application/json"
            """).strip())
        self.assertEqual(ret, 0)

    def test_spag_environment_crud(self):
        out, err, ret = run_spag('env', 'set', 'endpoint=abcdefgh')
        self.assertIn('endpoint: abcdefgh', out)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'show')
        self.assertIn('endpoint: abcdefgh', out)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'unset', '--everything')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'show')
        self.assertEqual(out, '{}\n\n')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

    def test_spag_environment_activate_deactivate(self):
        out, err, ret = run_spag('env', 'unset', '--everything')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'set', 'endpoint=abcdefgh')
        self.assertIn('endpoint: abcdefgh', out)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'deactivate')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('env', 'show')
        self.assertIn('endpoint: abcdefgh', out)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

    def test_spag_set_environment_failure(self):
        out, err, ret = run_spag('env', 'set')
        self.assertEqual(err, 'Error: You must provide something to set!\n')
        self.assertNotEqual(ret, 0)

    def test_set_endoint_and_header(self):
        out, err, ret = run_spag('env', 'set', 'endpoint=%s' % ENDPOINT, '-H', 'pglbutt:pglbutt')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertIn('headers', out)
        out, err, ret = run_spag('get', '/headers')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt"})


@unittest.skip("not implemented")
class TestSpagRemembers(BaseTest):

    def setUp(self):
        super(TestSpagRemembers, self).setUp()
        run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)

    def test_spag_remembers_request(self):
        auth_file = os.path.join(SPAG_REMEMBERS_DIR, 'v2/post_thing.yml')
        last_file = os.path.join(SPAG_REMEMBERS_DIR, 'last.yml')

        self.assertFalse(os.path.exists(SPAG_REMEMBERS_DIR))
        self.assertFalse(os.path.exists(auth_file))
        self.assertFalse(os.path.exists(last_file))

        _, err, ret = run_spag('request', 'v2/post_thing.yml',
                                 '--dir', RESOURCES_DIR)
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        self.assertTrue(os.path.exists(SPAG_REMEMBERS_DIR))
        self.assertTrue(os.path.exists(auth_file))
        self.assertTrue(os.path.exists(last_file))

        auth_data = files.load_file(auth_file)
        last_data = files.load_file(last_file)

        # check the saved request data
        req = auth_data['request']
        self.assertEqual(set(req.keys()),
            set(['body', 'endpoint', 'uri', 'headers', 'method']))
        self.assertEqual(req['method'], 'POST')
        self.assertEqual(req['endpoint'], ENDPOINT)
        self.assertEqual(req['uri'], '/things')
        self.assertEqual(req['headers']['Accept'], 'application/json')
        self.assertEqual(json.loads(req['body']), {"id": "c"})

        # check the saved response data
        resp = auth_data['response']
        self.assertEqual(set(resp.keys()), set(['body', 'headers', 'status']))
        self.assertEqual(resp['headers']['content-type'], 'application/json')
        self.assertEqual(resp['status'], 201)
        self.assertEqual(json.loads(resp['body']), {"id": "c"})

    def test_spag_remembers_get(self):
        self._test_spag_remembers_method_type('get')

    def test_spag_remembers_put(self):
        self._test_spag_remembers_method_type('put')

    def test_spag_remembers_post(self):
        self._test_spag_remembers_method_type('post')

    def test_spag_remembers_patch(self):
        self._test_spag_remembers_method_type('patch')

    def test_spag_remembers_delete(self):
        self._test_spag_remembers_method_type('delete')

    def _test_spag_remembers_method_type(self, method):
        filename = "{0}.yml".format(method)
        filepath = os.path.join(SPAG_REMEMBERS_DIR, filename)

        self.assertFalse(os.path.exists(filepath))

        _, err, ret = run_spag(method, '/poo', '--data', '{"id": "1"}')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        self.assertTrue(os.path.exists(filepath))

@unittest.skip("not implemented")
class TestSpagTemplate(BaseTest):

    def setUp(self):
        super(TestSpagTemplate, self).setUp()
        assert run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)[2] == 0
        assert run_spag('env', 'set', 'dir=%s' % TEMPLATES_DIR)

    def _post_thing(self, thing_id):
        """post a thing to set last.response.body.id"""
        out, err, ret = run_spag('post', '/things', '--data',
                                 '{"id": "%s"}' % thing_id,
                                 '-H', 'Content-type: application/json',
                                 '-H', 'Accept: application/json')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": thing_id})
        self.assertEqual(ret, 0)

    def test_spag_template_with_keyword(self):
        out, err, ret = run_spag('request', 'templates/post_thing',
                                 '--with', 'thing_id=wumbo')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "wumbo"})
        self.assertEqual(ret, 0)

    def test_spag_template_no_value_given(self):
        out, err, ret = run_spag('request', 'templates/post_thing')
        self.assertEqual(err, 'Failed to substitute for {{thing_id}}\n')
        self.assertEqual(out, '')
        self.assertEqual(ret, 1)

    def test_spag_template_empty_value(self):
        # we're allowed to substitute an empty string
        out, err, ret = run_spag('request', 'templates/post_thing',
                                 '--with', 'thing_id=')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": ""})
        self.assertEqual(ret, 0)

    def test_spag_template_multiple_with_keywords(self):
        out, err, ret = run_spag('request', 'templates/headers',
                                 '--with', 'hello=hello world',
                                 '--with', 'body_id=poo',
                                 '--with', 'thingy=my thing')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out),
            { "Body-Id": "poo",
              "Hello": "hello world",
              "Thingy": "my thing"  })
        self.assertEqual(ret, 0)

    def test_spag_template_alternative_items(self):
        # post a thing to set last.response.body.id
        self._post_thing('abcde')

        # the body-id in headers.yml is filled in using last.response.body.id
        # thingy is filled in using 'thingy2' insteada of 'thingy'
        out, err, ret = run_spag('request', 'templates/headers',
                                 '--with', 'hello=hello world',
                                 '--with', 'thingy2=scooby doo')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out),
            { "Body-Id": "abcde",
              "Hello": "hello world",
              "Thingy": "scooby doo" })
        self.assertEqual(ret, 0)

    def test_spag_template_alternative_items_with_overrides(self):
        # post a thing to set last.response.body.id
        self._post_thing('abcde')

        # we want to see that the body-id is taken from the --with arg and not
        # from last.response.body.id
        out, err, ret = run_spag('request', 'templates/headers',
                                 '--with', 'hello=hello world',
                                 '--with', 'body_id=wumbo',
                                 '--with', 'thingy2=scooby doo')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out),
            { "Body-Id": "wumbo",
              "Hello": "hello world",
              "Thingy": "scooby doo" })
        self.assertEqual(ret, 0)

    def test_spag_template_shortshortcut(self):
        # post a thing to set last.response.body.id
        self._post_thing('wumbo')

        out, err, ret = run_spag('get', '/things/@id')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "wumbo"})
        self.assertEqual(ret, 0)

    def test_spag_template_shortcut(self):
        # post a thing to set last.response.body.id
        self._post_thing('wumbo')

        out, err, ret = run_spag('get', '/things/@body.id')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "wumbo"})
        self.assertEqual(ret, 0)

    def test_spag_template_default(self):
        self._post_thing('mydefaultid')

        # with no --with args given, the get_default template should
        # default to "mydefaultid"
        out, err, ret = run_spag('request', 'template/get_default')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "mydefaultid"})
        self.assertEqual(ret, 0)

    def test_spag_template_shortcut_in_with(self):
        _, err, ret = run_spag('env', 'set', 'poke=pika')
        out, err, ret = run_spag('request', 'template/post_thing',
                                 '--with', 'thing_id=@[default].poke')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "pika"})
        self.assertEqual(ret, 0)

    def test_spag_template_shortcut_in_data(self):
        _, err, ret = run_spag('env', 'set', 'poke=pika')
        out, err, ret = run_spag('post', '/things',
                                 '--data', '{"id": "@[default].poke"}',
                                 '-H', 'Content-type: application/json',
                                 '-H', 'Accept: application/json')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "pika"})
        self.assertEqual(ret, 0)

    def test_spag_template_shortcut_in_header(self):
        _, err, ret = run_spag('env', 'set', 'poke=pika')
        out, err, ret = run_spag('get', '/headers', '-H', 'Poke: @[].poke')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"Poke": "pika"})
        self.assertEqual(ret, 0)

    def test_spag_template_from_default_and_active_environments(self):
        _, err, ret = run_spag('env', 'set',
                               'mini=barnacle boy',
                               'wumbo=mermaid man',
                               'thing=scooby doo')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('request', 'templates/get_default_env.yml')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out),
            { "Mini": "barnacle boy",
              "Wumbo": "mermaid man",
              "Thing": "scooby doo" })
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('request', 'templates/get_active_env.yml')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out),
            { "Mini": "barnacle boy",
              "Wumbo": "mermaid man",
              "Thing": "scooby doo" })
        self.assertEqual(ret, 0)

    def test_spag_template_shortcut_from_default_environment(self):
        _, err, ret = run_spag('request', 'template/post_thing',
                               '--with', 'thing_id=wumbo')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        # set thing_id and lookup thing_id in the env using shortcut syntax
        run_spag('env', 'set', 'thing_id=wumbo')
        out, err, ret = run_spag('get', '/things/@[default].thing_id')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), { "id": "wumbo" })
        self.assertEqual(ret, 0)

    def test_spag_template_list_indexing(self):
        # setup the last request to have a list in it
        self._post_thing('mini')
        _, err, ret = run_spag('get', '/things')

        out, err, ret = run_spag('get', '/things/@body.things.0.id')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"id": "mini"})
        self.assertEqual(ret, 0)

    def test_spag_template_list_index_out_of_bounds(self):
        # setup the last request to have a list in it
        self._post_thing('mini')
        _, err, ret = run_spag('get', '/things')

        out, err, ret = run_spag('get', '/things/@body.things.1.id')
        self.assertIn('Index 1 out of bounds while looking up response.body.things.1.id', err)
        self.assertEqual(ret, 1)

    def test_spag_template_list_w_invalid_index(self):
        # setup the last request to have a list in it
        self._post_thing('mini')
        _, err, ret = run_spag('get', '/things')

        out, err, ret = run_spag('get', '/things/@body.things.poo.id')
        self.assertIn('Invalid list index poo while fetching response.body.things.poo.id', err)
        self.assertEqual(ret, 1)

    def test_spag_templated_env_set(self):
        # set a value we'll refer to in a template parameter
        out, err, ret = run_spag('env', 'set', 'squidward=tentacle')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(yaml.load(out).get('squidward'), 'tentacle')

        # check we can use template params with regular environment variables
        out, err, ret = run_spag('env', 'set', 'patrick=@[].squidward')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(yaml.load(out).get('patrick'), 'tentacle')

        # check we can use template params when setting headers in the env
        out, err, ret = run_spag('env', 'set', '-H', 'sandy: @[].patrick')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertEqual(yaml.load(out)['headers'].get('sandy'), 'tentacle')


@unittest.skip("not implemented")
class TestSpagHistory(BaseTest):

    def setUp(self):
        super(TestSpagHistory, self).setUp()
        _, _, ret = run_spag('env', 'set', 'endpoint=%s' % ENDPOINT)
        _, _, ret = run_spag('env', 'set', 'dir=%s' % TEMPLATES_DIR)
        self.assertEqual(ret, 0)

    def test_empty_history(self):
        out, err, ret = run_spag('history')

        self.assertEqual(err, '')
        self.assertEqual(out.strip(), '')
        self.assertEqual(ret, 0)

    def _run_test_method_history(self, spag_call, expected):
        _, err, ret = spag_call()
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('history')
        self.assertEqual(err, '')
        self.assertEqual(out, expected)

    def test_post_method_history(self):
        self._run_test_method_history(
            lambda: run_spag('post', '/things', '--data={"id": "posty"}'),
            expected='0: POST %s/things\n' % ENDPOINT)

    def test_put_method_history(self):
        self._run_test_method_history(
            lambda: run_spag('put', '/things/id'),
            expected='0: PUT %s/things/id\n' % ENDPOINT)

    def test_patch_method_history(self):
        self._run_test_method_history(
            lambda: run_spag('patch', '/things/id'),
            expected='0: PATCH %s/things/id\n' % ENDPOINT)

    def test_get_method_history(self):
        self._run_test_method_history(
            lambda: run_spag('get', '/things/wumbo'),
            expected='0: GET %s/things/wumbo\n' % ENDPOINT)

    def test_delete_method_history(self):
        self._run_test_method_history(
            lambda: run_spag('delete', '/things/wumbo'),
            expected='0: DELETE %s/things/wumbo\n' % ENDPOINT)

    def test_multi_history_items(self):
        # make three requests
        _, err, _ = run_spag('get', '/things')
        self.assertEqual(err, '')
        _, err, _ = run_spag('request', 'template/post_thing',
                             '--with', 'thing_id=wumbo')
        self.assertEqual(err, '')
        _, err, _ = run_spag('get', '/things/wumbo')
        self.assertEqual(err, '')

        # check `spag history`
        out, err, ret = run_spag('history')
        self.assertEqual(err, '')
        self.assertEqual(out,
            "0: GET {0}/things/wumbo\n"
            "1: POST {0}/things\n"
            "2: GET {0}/things\n"
            .format(ENDPOINT))
        self.assertEqual(ret, 0)

        # check 'spag history show'
        out, err, ret = run_spag('history', 'show', '1')
        self.assertEqual(err, '')
        self.assertIn('POST %s/things' % ENDPOINT, out)
        self.assertEqual(ret, 0)

