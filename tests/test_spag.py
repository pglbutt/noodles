import unittest
import subprocess
import os

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
    return (out, err, p.returncode)


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
