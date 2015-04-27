import unittest
import subprocess
import os
import json

import spag_files

# TODO: read this from a config?
SPAG_PROG = 'spag'
ENDPOINT = 'http://localhost:5000'
RESOURCES_DIR = os.path.join(os.path.dirname(__file__), 'resources')
V1_RESOURCES_DIR = os.path.join(RESOURCES_DIR, 'v1')
V2_RESOURCES_DIR = os.path.join(RESOURCES_DIR, 'v2')

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

    def setUp(self):
        run_spag('get', '/clear', '-e', ENDPOINT)
        run_spag('clear')


class TestEnvironment(BaseTest):

    def test_spag_endpoint_crud(self):
        out, err, ret = run_spag('set', 'abcdefgh')
        self.assertEqual(out, 'endpoint: abcdefgh\n')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('show')
        self.assertEqual(out, 'endpoint: abcdefgh\n')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('clear')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('show')
        self.assertEqual(out, '')
        self.assertEqual(err, 'Endpoint not set\n')
        self.assertNotEqual(ret, 0)

    def test_spag_set_environment_failure(self):
        out, err, ret = run_spag('set')
        self.assertEqual(err, 'Error: You must provide something to set!\n')
        self.assertNotEqual(ret, 0)

    def test_set_endoint_and_header(self):
        out, err, ret = run_spag('set', ENDPOINT, '-H', 'pglbutt:pglbutt')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        self.assertIn('headers', out)
        out, err, ret = run_spag('get', '/headers')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt"})


class TestHeaders(BaseTest):

    def test_get_no_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {})

    def test_get_one_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H', 'pglbutt:pglbutt')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt"})

    def test_get_two_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT,
                                 '-H', 'pglbutt:pglbutt', '-H', 'wow:wow')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt", "Wow": "wow"})

    def test_get_no_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: -H option requires an argument\n')

    def test_get_invalid_header(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-H', 'poo')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: Invalid header!\n')

    def test_show_headers(self):
        out, err, ret = run_spag('get', '/headers', '-e', ENDPOINT, '-h')
        self.assertEqual(ret, 0)
        self.assertIn('content-type: application/json', out)


class TestGet(BaseTest):

    def test_get_no_endpoint(self):
        out, err, ret = run_spag('get', '/auth')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Endpoint not set\n')

    def test_get_supply_endpoint(self):
        out, err, ret = run_spag('get', '/auth', '-e', ENDPOINT)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"token": "abcde"})

    def test_get_presupply_endpoint(self):
        out, err, ret = run_spag('set', ENDPOINT)
        self.assertEqual(out, 'endpoint: {0}\n'.format(ENDPOINT))
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        out, err, ret = run_spag('get', '/things')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"things": []})


class TestPost(BaseTest):

    def test_spag_post(self):
        run_spag('set', ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')

class TestPut(BaseTest):

    def test_spag_put(self):
        run_spag('set', ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('put', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "b"})
        self.assertEquals(err, '')

class TestPatch(BaseTest):

    def test_spag_patch(self):
        run_spag('set', ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('patch', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "b"})
        self.assertEquals(err, '')

class TestDelete(BaseTest):

    def test_spag_delete(self):
        run_spag('set', ENDPOINT)
        out, err, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(json.loads(out), {"id": "a"})
        self.assertEquals(err, '')
        out, err, ret = run_spag('delete', '/things/a', '--data', '{"id": "b"}',
                                 '-H', 'content-type: application/json')
        self.assertEquals(ret, 0)
        self.assertEquals(out, '\n')
        self.assertEquals(err, '')
        out, err, ret = run_spag('get', '/things')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"things": []})


class TestSpagFiles(BaseTest):

    def setUp(self):
        super(TestSpagFiles, self).setUp()
        run_spag('set', ENDPOINT)
        self.table = spag_files.SpagFilesLookup(RESOURCES_DIR)

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
        content = spag_files.load_file(os.path.join(RESOURCES_DIR, 'auth.yml'))
        self.assertEqual(content['method'], 'GET')
        self.assertEqual(content['uri'], '/auth')
        self.assertEqual(content['headers'], {'Accept': 'application/json'})

    def test_spag_request_get(self):
        out, err, ret = run_spag('request', 'auth.yml', '--dir', RESOURCES_DIR)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"token": "abcde"})
        self.assertEqual(err, '')

    def test_spag_request_post(self):
        out, err, ret = run_spag('request', 'v2/post_thing.yml',
                                 '--dir', RESOURCES_DIR)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"id": "c"})
        self.assertEqual(err, '')

    def test_spag_request_patch(self):
        # stuff in patch_thing.yml needs to match stuff here
        _, _, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                              '-H', 'content-type: application/json')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('request', 'patch_thing.yml',
                                 '--dir', RESOURCES_DIR)
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"id": "c"})
        self.assertEqual(err, '')

    def test_spag_request_delete(self):
        _, _, ret = run_spag('post', '/things', '--data', '{"id": "a"}',
                              '-H', 'content-type: application/json')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('request', 'delete_thing.yml',
                                 '--dir', RESOURCES_DIR)
        self.assertEqual(ret, 0)
        self.assertEqual(out, '\n')
        self.assertEqual(err, '')

    def test_spag_request_data_option_overrides(self):
        out, err, ret = run_spag('request', 'v2/post_thing.yml',
                                 '--data', '{"id": "xyz"}',
                                 '--dir', RESOURCES_DIR,
                                 '-H', 'content-type: application/json')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"id": "xyz"})
        self.assertEqual(err, '')

    def test_spag_request_headers_override(self):
        out, err, ret = run_spag('request', 'headers.yml',
                                 '--dir', RESOURCES_DIR,
                                 '-H', 'Hello: abcde')
        self.assertEqual(err, '')
        self.assertEqual(json.loads(out), {"Hello": "abcde"})
        self.assertEqual(ret, 0)
