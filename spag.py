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

def fix_endpoint(f):
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
@click.argument('endpoint', default=None, required=False)
@fix_endpoint
def get(resource, endpoint=None):
    """HTTP GET"""
    uri = endpoint+resource
    r = requests.get(uri)
    click.echo(json.dumps(r.json(), indent=4))

if __name__ == '__main__':
    cli()
