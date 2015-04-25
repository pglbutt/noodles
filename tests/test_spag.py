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
        out, err, ret = run_spag('get', '/auth', 'http://localhost:5000')
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

