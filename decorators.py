import sys
import functools

import click

import spag_files
from common import ToughNoodles, update

def determine_endpoint(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = spag_files.SpagEnvironment.get_env()
        if kwargs['endpoint'] is None:
            try:
                endpoint = env['envvars']['endpoint']
                kwargs['endpoint'] = endpoint
            except ToughNoodles as e:
                click.echo(str(e), err=True)
                sys.exit(1)
            except KeyError:
                click.echo('Endpoint not set\n', err=True)
                sys.exit(1)
        return f(*args, **kwargs)
    return wrapper

def prepare_headers(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = spag_files.SpagEnvironment.get_env()
        header = kwargs['header']
        if header is None or header == ():
            try:
                headers = env['headers']
                kwargs['header'] = headers
            except ToughNoodles:
                kwargs['header'] = None
            except KeyError:
                kwargs['header'] = None
        else:
            try:
                # Headers come in as a tuple ('Header:Content', 'Header:Content')
                supplied_headers = {key: value.strip() for (key, value) in [h.split(':') for h in header]}
                if 'headers' in env:
                    headers = update(env['headers'], supplied_headers)
                else:
                    headers = supplied_headers
                kwargs['header'] = headers
            except ValueError:
                click.echo("Error: Invalid header!", err=True)
                sys.exit(1)
            except KeyError:
                kwargs['header'] = {key: value.strip() for (key, value) in [h.split(':') for h in header]}

        return f(*args, **kwargs)
    return wrapper

def request_dir(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        env = spag_files.SpagEnvironment.get_env()

        if kwargs['dir'] is None:
            try:
                d = env['envvars']['dir']
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