import os
import string

import yaml
import click

from spag import files
from spag import common

def resp_to_dict(resp):
    result = {}

    # for url = 'http://example.com/a/b', we want to separate
    # 'http://example.com' and '/a/b'
    assert resp.request.url.endswith(resp.request.path_url)
    endpoint = resp.request.url.rsplit(resp.request.path_url, 1)[0]

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
        'body': resp.text,
    }
    return result


class SpagRemembers(files.SpagFilesLookup):

    DIR = './.spag/remembers/'

    def __init__(self):
        common.ensure_dir_exists(self.DIR)
        super(SpagRemembers, self).__init__(self.DIR)

    @classmethod
    def remember_request(cls, name, resp):
        """Save the request and response in resp to a file

        :param name: The filename, optionally without the .yml extension.
        :param resp: A requests response object
        """
        common.ensure_dir_exists(cls.DIR)

        # always save the last request as 'last' in addition to any other name
        if name != 'last':
            SpagRemembers.remember_request('last', resp)

        result = resp_to_dict(resp)

        path_parts = common.split_path(name)

        # for a/b/c.yml, ensure we have the .yml extension
        filename = path_parts[-1]
        if not files.SpagFilesLookup.has_valid_extension(filename):
            filename += files.SpagFilesLookup.VALID_EXTENSION

        # for a/b/c.yml, put it in <dir>/a/b/c.yml (and not <dir>/c.yml)
        filedir = cls.DIR
        if len(path_parts) > 1:
            filedir = os.path.join(cls.DIR, *path_parts[:-1])
        common.ensure_dir_exists(filedir)

        filename = os.path.join(filedir, filename)
        with open(filename, 'w') as f:
            yaml.safe_dump(result, f, default_flow_style=False)


class SpagHistory(object):

    FILENAME = '.spag/history.yml'

    """The number of requests to store in the history file"""
    SIZE = 1000

    @classmethod
    def short_output(cls, entry):
        return "%s %s%s" % (entry['request']['method'],
                            entry['request']['endpoint'],
                            entry['request']['uri'])

    @classmethod
    def long_output(cls, entry):
        result = "{:-^50}\n".format(" Request ")
        result += "%s %s%s\n" % (entry['request']['method'],
                                 entry['request']['endpoint'],
                                 entry['request']['uri'])
        for k, v in entry['request']['headers'].items():
            result += '%s: %s\n' % (k, v)
        if entry['request']['body']:
            result += entry['request']['body']

        result += "{:-^50}\n".format(" Response ")
        result += "Status code %s\n" % entry['response']['status']
        for k, v in entry['response']['headers'].items():
            result += '%s: %s\n' % (k, v)
        result += entry['response']['body']

        return result

    @classmethod
    def _read_file(cls):
        if os.path.exists(cls.FILENAME):
            with open(cls.FILENAME, 'r') as f:
                return yaml.safe_load(f)
        return []

    @classmethod
    def get(cls, index):
        # convert to int, error handling
        try:
            index = int(index)
        except ValueError as e:
            raise common.ToughNoodles("Invalid history index %s" % index)
        if index < 0:
            raise common.ToughNoodles("Invalid history index %s" % index)

        history = cls._read_file()
        if index < len(history):
            return history[index]
        raise common.ToughNoodles(
            "ERROR: History index %s out of bounds. Only %s entries."
            % (index, len(history)))

    @classmethod
    def list(cls):
        history = cls._read_file()
        for i, h in enumerate(history):
            click.echo("%s: %s" % (i, cls.short_output(h)))

    @classmethod
    def show(cls, index):
        entry = cls.get(index)
        click.echo(cls.long_output(entry))

    @classmethod
    def append(cls, resp):
        entry = resp_to_dict(resp)
        history = cls._read_file()
        if len(history) == cls.SIZE:
            history = [entry] + history[:cls.SIZE - 1]
        else:
            history = [entry] + history
        with open(cls.FILENAME, 'w') as f:
            yaml.safe_dump(history, f, default_flow_style=False)
