import sys
import functools

import click

from spag import files
from spag.common import ToughNoodles

def determine_endpoint(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = files.SpagEnvironment.get_env()
        if kwargs['endpoint'] is None:
            try:
                endpoint = env['endpoint']
                kwargs['endpoint'] = endpoint
            except ToughNoodles as e:
                click.echo(str(e), err=True)
                sys.exit(1)
            except KeyError:
                click.echo('Endpoint not set\n', err=True)
                sys.exit(1)
        return f(*args, **kwargs)
    return wrapper

def _headers_to_dict(headers):
    if type(headers) != dict:
        # assume something iterable, like tuple or list
        return {key: value.strip()
                for (key, value) in [h.split(':') for h in headers]}
    return headers

def prepare_headers(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = files.SpagEnvironment.get_env()
        try:
            if 'headers' in env:
                # should already be a dict...
                env['headers'] = _headers_to_dict(env['headers'])
            if 'header' in kwargs:
                kwargs['header'] = _headers_to_dict(kwargs['header'])
                kwargs['header'].update(env.get('headers', {}))
            else:
                kwargs['header'] = dict(env.get('headers', {}))
        except ValueError:
            click.echo("Error: Invalid header!", err=True)
            sys.exit(1)
        assert type(kwargs['header']) is dict
        return f(*args, **kwargs)
    return wrapper

def request_dir(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = files.SpagEnvironment.get_env()

        if kwargs['dir'] is None:
            try:
                d = env['dir']
                kwargs['dir'] = d
            except ToughNoodles as e:
                click.echo(str(e), err=True)
                sys.exit(1)
            except KeyError:
                click.echo('Request directory not set\n', err=True)
                sys.exit(1)
        return f(*args, **kwargs)
    return wrapper

def common_request_args(f):
    @click.option('--endpoint', '-e', metavar='<endpoint>',
                  default=None, help='Manually override the endpoint')
    @click.option('--header', '-H', metavar = '<header>', multiple=True,
                  default=None, help='Header in the form Key:Value')
    @click.option('--show-headers', '-h', is_flag=True,
                  help='Prints the headers along with the response body')
    @click.option('--data', '-d', required=False, help='the request data')
    @determine_endpoint
    @prepare_headers
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        return f(*args, **kwargs)
    return wrapper
