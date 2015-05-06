#!/usr/bin/env python

import os
import sys
import json
import functools

import click
import requests
import yaml

import spag_files
from spag_remembers import SpagRemembers
from common import ToughNoodles, update

def determine_endpoint(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        if kwargs['endpoint'] is None:
            try:
                endpoint = spag_files.SpagEnvironment.get_env()['endpoint']
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
        header = kwargs['header']
        if header is None or header == ():
            try:
                headers = spag_files.SpagEnvironment.get_env()['headers']
                kwargs['header'] = headers
            except ToughNoodles:
                kwargs['header'] = None
            except KeyError:
                kwargs['header'] = None
        else:
            try:
                # Headers come in as a tuple ('Header:Content', 'Header:Content')
                env = spag_files.SpagEnvironment.get_env()
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
        if kwargs['dir'] is None:
            try:
                d = spag_files.SpagEnvironment.get_env()['envvars']['dir']
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


@click.group()
@click.version_option()
def cli():
    """Spag.

    This is the spag http client. It's spagtacular.
    """

def show_response(resp, show_headers):
    if show_headers:
        for k, v in resp.headers.items():
            click.echo("{0}: {1}".format(k, v))
    click.echo(resp.text)

@cli.command('get')
@click.argument('resource')
@common_request_args
def get(resource, endpoint=None, data=None, header=None, show_headers=False):
    """HTTP GET"""
    uri = endpoint + resource
    r = requests.get(uri, headers=header, data=data)
    show_response(r, show_headers)
    SpagRemembers.remember_request('get', r)

@cli.command('post')
@click.argument('resource')
@common_request_args
def post(resource, endpoint=None, data=None, header=None, show_headers=False):
    """HTTP POST"""
    uri = endpoint + resource
    r = requests.post(uri, data=data, headers=header)
    show_response(r, show_headers)
    SpagRemembers.remember_request('post', r)

@cli.command('put')
@click.argument('resource')
@common_request_args
def put(resource, endpoint=None, data=None, header=None, show_headers=False):
    """HTTP PUT"""
    uri = endpoint + resource
    r = requests.put(uri, data=data, headers=header)
    show_response(r, show_headers)
    SpagRemembers.remember_request('put', r)

@cli.command('patch')
@click.argument('resource')
@common_request_args
def patch(resource, endpoint=None, data=None, header=None, show_headers=False):
    """HTTP PATCH"""
    uri = endpoint + resource
    r = requests.patch(uri, data=data, headers=header)
    show_response(r, show_headers)
    SpagRemembers.remember_request('patch', r)

@cli.command('delete')
@click.argument('resource')
@common_request_args
def delete(resource, endpoint=None, data=None, header=None, show_headers=False):
    """HTTP DELETE"""
    uri = endpoint + resource
    r = requests.delete(uri, data=data, headers=header)
    show_response(r, show_headers)
    SpagRemembers.remember_request('delete', r)


@cli.command('request')
@click.argument('name', required=False)
@common_request_args
@request_dir
@click.option('--dir', required=False,
              help='the dir to search for request files')
@click.option('--show', required=False, is_flag=True,
              help='show request file, or show all request files if no name')
def request(dir=None, name=None, endpoint=None, data=None, header=None,
            show_headers=False, show=False):
    try:
        if show and name is None:
            for x in spag_files.SpagFilesLookup(dir).get_file_list():
                click.echo(x)
        elif show:
            filename = spag_files.SpagFilesLookup(dir).get_path(name)
            filename = os.path.relpath(filename, '.')
            click.echo("File {0}".format(filename))
            with click.open_file(filename, 'r') as f:
                click.echo(f.read())
            # maybe should we still perform the request?
        else:
            filename = spag_files.SpagFilesLookup(dir).get_path(name)

            # load the request data into a dict
            req = spag_files.load_file(filename)
            kwargs = {
                'url': endpoint + req['uri'],
                'headers': header or req.get('headers', {})
            }
            if data is not None:
                kwargs['data'] = data
            elif 'body' in req:
                kwargs['data'] = req['body']

            # I don't know how to call click-decorated get(), post(), etc functions
            # Use requests directly instead
            method = req['method'].lower()
            resp = getattr(requests, method)(**kwargs)
            show_response(resp, show_headers)

            SpagRemembers.remember_request(name, resp)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@cli.group('env')
def env():
    """Spag environments"""

@env.command('activate')
@click.argument('envname', required=True)
def env_activate(envname):
    try:
        spag_files.SpagEnvironment().activate(envname)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)
    click.echo('Environment %s activated' % envname)

@env.command('deactivate')
def env_deactivate():
    try:
        spag_files.SpagEnvironment().deactivate()
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)
    click.echo('Deactivated')

@env.command('show')
@click.argument('envname', required=False)
def env_show(envname=None):
    try:
        env = spag_files.SpagEnvironment().get_env(envname)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@env.command('set')
@click.argument('endpoint', default=None, required=False)
@click.option('--header', '-H', multiple=True,
              default=None, help='Header in the form key:value')
@click.option('--envvars', '-E', multiple=True,
              default=None, help='Environment variables in the form key=value')
def env_set(endpoint=None, header=None, envvars=None):
    """Set the endpoint and/or headers."""
    if endpoint is None and header == () and envvars == ():
        click.echo("Error: You must provide something to set!", err=True)
        sys.exit(1)

    # Switch envvars, headers from Tuples to dict
    envvars = {key: value for (key, value) in [e.split('=') for e in envvars]}
    header = {key: value.strip() for (key, value) in [h.split(':') for h in header]}

    # Determine which args should be passed to a dict-style update function
    kwargs = {'endpoint': endpoint, 'headers': header, 'envvars': envvars}
    for arg in ['endpoint', 'headers', 'envvars']:
        if not kwargs[arg]:
            kwargs.pop(arg)

    try:
        env = spag_files.SpagEnvironment().set_env(kwargs)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@env.command('unset')
@click.argument('resource', required=True)
@click.option('--everything', is_flag=True, default=False)
def env_unset(resource, everything=False):
    try:
        env = spag_files.SpagEnvironment().unset_env(resource, everything)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

if __name__ == '__main__':
    cli()
