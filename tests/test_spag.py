import unittest
import subprocess
import os
import json

SPAG_PROG = 'spag'

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


class TestSpag(unittest.TestCase):

    def test_spag_endpoint_crud(self):
        out, err, ret = run_spag('set', 'abcdefgh')
        self.assertEqual(out, 'abcdefgh\n')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)

        out, err, ret = run_spag('show')
        self.assertEqual(out, 'abcdefgh\n')
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
        run_spag('clear')
        out, err, ret = run_spag('set')
        self.assertTrue(err.startswith('Usage:'))
        self.assertNotEqual(ret, 0)

    # HTTP GET Tests
    def test_get_no_endpoint(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/auth')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Endpoint not set\n')

    def test_get_supply_endpoint(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/auth', '-e', 'http://localhost:5000')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"token": "abcde"})

    def test_get_presupply_endpoint(self):
        run_spag('clear')
        out, err, ret = run_spag('set', 'http://localhost:5000')
        self.assertEqual(out, 'http://localhost:5000\n')
        self.assertEqual(err, '')
        self.assertEqual(ret, 0)
        out, err, ret = run_spag('get', '/things')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"things": [{"id": "1"}]})

    # Headers Tests
    def test_get_no_headers(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {})

    def test_get_one_header(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000', '-H', 'pglbutt:pglbutt')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt"})

    def test_get_two_headers(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000',
                                 '-H', 'pglbutt:pglbutt', '-H', 'wow:wow')
        self.assertEqual(ret, 0)
        self.assertEqual(json.loads(out), {"Pglbutt": "pglbutt", "Wow": "wow"})

    def test_get_no_header(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000', '-H')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: -H option requires an argument\n')

    def test_get_invalid_header(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000', '-H', 'poo')
        self.assertNotEqual(ret, 0)
        self.assertEqual(err, 'Error: Invalid header!\n')

    def test_show_headers(self):
        run_spag('clear')
        out, err, ret = run_spag('get', '/headers', '-e', 'http://localhost:5000', '-h')
        self.assertEqual(ret, 0)
        self.assertIn('content-type: application/json', out)

