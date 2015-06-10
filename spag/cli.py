#!/usr/bin/env python

import os
import sys
import json
import functools

import click
import requests
import yaml

from spag import files
from spag import template
from spag import remembers
from spag import decorators as dec
from spag.common import ToughNoodles


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
    if resp.ok:
        click.echo(resp.text)
    else:
        click.echo("ERROR: %s %s" % (str(resp.status_code), resp.reason))
        click.echo(resp.text)

def do_request(method, resource, endpoint=None, header=None, data=None,
               show_headers=False, remember_as=None):

    if data: data = template.untemplate(data, shortcuts=True)
    if header: header = {k: template.untemplate(v, shortcuts=True) for k, v in header.items()}

    req = requests.Request(
        method = method,
        url = template.untemplate(endpoint + resource, shortcuts=True),
        headers=header,
        data=data)
    req = req.prepare()

    resp = requests.Session().send(req)
    show_response(resp, show_headers)

    remember_as = remember_as or method.lower()
    remembers.SpagRemembers.remember_request(remember_as, resp)
    remembers.SpagHistory.append(resp)

@cli.command('get')
@click.argument('resource')
@dec.common_request_args
def get(*args, **kwargs):
    """HTTP GET"""
    do_request('GET', *args, **kwargs)

@cli.command('post')
@click.argument('resource')
@dec.common_request_args
def post(*args, **kwargs):
    """HTTP POST"""
    do_request('POST', *args, **kwargs)

@cli.command('put')
@click.argument('resource')
@dec.common_request_args
def put(*args, **kwargs):
    """HTTP PUT"""
    do_request('PUT', *args, **kwargs)

@cli.command('patch')
@click.argument('resource')
@dec.common_request_args
def patch(*args, **kwargs):
    """HTTP PATCH"""
    do_request('PATCH', *args, **kwargs)

@cli.command('delete')
@click.argument('resource')
@dec.common_request_args
def delete(*args, **kwargs):
    """HTTP DELETE"""
    do_request('DELETE', *args, **kwargs)

@cli.command('request')
@click.argument('name', required=False)
@dec.common_request_args
@dec.request_dir
@click.option('--dir', required=False,
              help='the dir to search for request files')
@click.option('--show', required=False, is_flag=True,
              help='show request file, or show all request files if no name')
@click.option('withs', '--with', '-w', metavar = '<with>', multiple=True,
              default=[], help='specify values for vars in your request templates')
def request(dir=None, name=None, endpoint=None, data=None, header=None,
            show_headers=False, show=False, withs=None):
    try:
        if show and name is None:
            for x in files.SpagFilesLookup(dir).get_file_list():
                click.echo(x)
        elif show:
            filename = files.SpagFilesLookup(dir).get_path(name)
            filename = os.path.relpath(filename, '.')
            click.echo("File {0}".format(filename))
            # TODO: show the untemplated version of the file here
            with click.open_file(filename, 'r') as f:
                click.echo(f.read())
            # maybe should we still perform the request?
        else:
            filename = files.SpagFilesLookup(dir).get_path(name)

            with open(filename, 'r') as f:
                raw = template.untemplate(f.read(), withs)

            req = yaml.safe_load(raw)

            kwargs = {
                'method': req['method'].upper(),
                'endpoint': endpoint,
                'resource': req['uri'],
                'header': header or req.get('headers', {}),
                'show_headers': show_headers,
                'remember_as': name,
            }
            if data is not None:
                kwargs['data'] = template.untemplate(data, shortcuts=True)
            elif 'body' in req:
                kwargs['data'] = req['body']

            do_request(**kwargs)
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
        files.SpagEnvironment().activate(envname)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)
    click.echo('Environment %s activated' % envname)

@env.command('deactivate')
def env_deactivate():
    try:
        files.SpagEnvironment().deactivate()
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)
    click.echo('Deactivated')

@env.command('show')
@click.argument('envname', required=False)
def env_show(envname=None):
    try:
        env = files.SpagEnvironment().get_env(envname)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@env.command('set')
@click.argument('envvars', nargs=-1, required=False)
@click.option('--header', '-H', multiple=True,
              default=None, help='Header in the form key:value')
def env_set(envvars=None, header=None):
    """Set the environment variables and/or headers."""
    if header == () and envvars == ():
        click.echo("Error: You must provide something to set!", err=True)
        sys.exit(1)

    # Switch envvars, headers from Tuples to dict
    envvars = {key: template.untemplate(value, shortcuts=True)
               for (key, value) in [e.split('=') for e in envvars]}
    header = {key: template.untemplate(value.strip(), shortcuts=True)
              for (key, value) in [h.split(':') for h in header]}

    if header:
        envvars['headers'] = header

    try:
        env = files.SpagEnvironment().set_env(envvars)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@env.command('unset')
@click.argument('resource', required=False)
@click.option('--everything', is_flag=True, default=False)
def env_unset(resource=None, everything=False):
    try:
        env = files.SpagEnvironment().unset_env(resource, everything)
        click.echo(yaml.safe_dump(env, default_flow_style=False))
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@cli.group('history', invoke_without_command=True)
@click.pass_context
def history(ctx):
    """Show request history"""
    # do this on `spag history`
    # don't do this on `spag history <cmd>`
    if ctx.invoked_subcommand is None:
        try:
            remembers.SpagHistory.list()
        except ToughNoodles as e:
            click.echo(str(e), err=True)
            sys.exit(1)

@history.command('show')
@click.argument('index', required=True)
def show_history_entry(index):
    try:
        remembers.SpagHistory.show(index)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)
