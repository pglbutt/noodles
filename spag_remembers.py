import os
import errno

import spag_files
import common

import yaml

def ensure_dir_exists(dir):
    try:
        os.makedirs(dir)
    except OSError as e:
        # if dir already exists, we're good to go
        if e.errno == errno.EEXIST and os.path.isdir(dir):
            return
        raise common.ToughNoodles(str(e))

class SpagRemembers(spag_files.SpagFilesLookup):

    DIR = './.spag/remembers/'

    def __init__(self):
        ensure_dir_exists(self.DIR)
        super(SpagRemembers, self).__init__(self.DIR)

    @classmethod
    def remember_request(cls, name, resp):
        """Save the request and response in resp to a file

        :param name: The filename, optionally without the .yml extension.
        :param resp: A requests response object
        """
        ensure_dir_exists(cls.DIR)

        # always save the last request as 'last
        if name != 'last':
            SpagRemembers.remember_request('last', resp)

        # for url = 'http://example.com/a/b', we want to separate
        # 'http://example.com' and '/a/b'
        assert resp.request.url.endswith(resp.request.path_url)
        endpoint = resp.request.url.rsplit(resp.request.path_url, 1)[0]

        result = {}
        result['request'] = {
            'method': resp.request.method,
            'endpoint': endpoint,
            'headers': dict(resp.request.headers),
            'uri': resp.request.path_url,
            'body': resp.request.body,
        }
        result['response'] = {
            'status': resp.status_code,
            'headers': dict(resp.headers),
            'body': resp.request.body,
        }

        path_parts = common.split_path(name)

        # for a/b/c.yml, ensure we have the .yml extension
        filename = path_parts[-1]
        if not spag_files.SpagFilesLookup.has_valid_extension(filename):
            filename += spag_files.SpagFilesLookup.VALID_EXTENSION

        # for a/b/c.yml, put it in <dir>/a/b/c.yml (and not <dir>/c.yml)
        filedir = cls.DIR
        if len(path_parts) > 1:
            filedir = os.path.join(cls.DIR, *path_parts[:-1])
        ensure_dir_exists(filedir)

        filename = os.path.join(filedir, filename)
        with open(filename, 'w') as f:
            yaml.safe_dump(result, f, default_flow_style=False)
