#!/usr/bin/env python

import os
import sys
import json
import functools

import click
import requests

class ToughNoodles(Exception):
    pass

class Endpoint(object):

    ENDPOINT_FILE = '.spag.endpoint'
    @classmethod
    def get(cls):
        if os.path.exists(cls.ENDPOINT_FILE):
            with click.open_file(cls.ENDPOINT_FILE, 'r') as f:
                return f.read()
        else:
            raise ToughNoodles("Endpoint not set")

    @classmethod
    def set(cls, endpoint):
        with click.open_file(cls.ENDPOINT_FILE, 'w') as f:
            f.write(endpoint)

    @classmethod
    def clear(cls):
        if os.path.exists(cls.ENDPOINT_FILE):
            os.remove(cls.ENDPOINT_FILE)

def determine_endpoint(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        if kwargs['endpoint'] is None:
            try:
                endpoint = Endpoint.get()
                kwargs['endpoint'] = endpoint
            except ToughNoodles as e:
                click.echo(str(e), err=True)
                sys.exit(1)
        return f(*args, **kwargs)
    return wrapper

def prepare_headers(f):
    @functools.wraps(f)
    def wrapper(*args, **kwargs):
        header = kwargs['header']
        if header is None:
            kwargs['header'] = {}
        else:
            try:
                # Headers come in as a tuple ('Header:Content', 'Header:Content')
                kwargs['header'] = {key: value for (key, value) in [h.split(':') for h in header]}
            except ValueError:
                click.echo("Error: Invalid header!", err=True)
                sys.exit(1)

        return f(*args, **kwargs)
    return wrapper

@click.group()
@click.version_option()
def cli():
    """Spag.

    This is the spag http client. It's spagtacular.
    """

@cli.command('set')
@click.argument('endpoint', default=None)
def spag_set(endpoint=None):
    """Set the endpoint."""
    try:
        Endpoint.set(endpoint)
        click.echo(endpoint)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@cli.command('show')
def spag_show():
    """Show the endpoint."""
    try:
        endpoint = Endpoint.get()
        click.echo(endpoint)
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@cli.command('clear')
def spag_clear():
    """Clear the endpoint."""
    try:
        Endpoint.clear()
        click.echo("Endpoint cleared.")
    except ToughNoodles as e:
        click.echo(str(e), err=True)
        sys.exit(1)

@cli.command('get')
@click.argument('resource')
@click.option('--endpoint', '-e', metavar='<endpoint>',
              default=None, help='Manually override the endpoint')
@click.option('--header', '-H', metavar = '<header>', multiple=True,
              default=None, help='Header in the form Key:Value')
@click.option('--show-headers', '-h', is_flag=True,
              help='Prints the headers along with the response body')
@determine_endpoint
@prepare_headers
def get(resource, endpoint=None, header=None, show_headers=False):
    """HTTP GET"""
    uri = endpoint + resource

    r = requests.get(uri, headers=header)

    if show_headers:
        for header, value in r.headers.items():
            click.echo("%s: %s" % (header, value))

    click.echo("\n%s" % r.text)

if __name__ == '__main__':
    cli()
