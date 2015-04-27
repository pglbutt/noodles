import unittest
import subprocess
import os
import json

SPAG_PROG = 'spag'
ENDPOINT = 'http://localhost:5000'

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


class TestEndpoint(BaseTest):

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

    def test_spag_set_endpoint_failure(self):
        out, err, ret = run_spag('set')
        self.assertEqual(err, 'Error: You must provide something to set!\n')
        self.assertNotEqual(ret, 0)


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

    def test_spag_post(self):
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

