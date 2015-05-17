from setuptools import setup

setup(name='noodles',
      version='0.0.1',
      description='The best command line HTTP client ever',
      url='http://github.com/pglbutt/noodles',
      author='pglbutt',
      author_email='pglbutt@pglbutt.com',
      license='MIT',
      py_modules=['spag', 'spag_files', 'spag_remembers', 'common', 'spag_template', 'decorators'],
      install_requires=[
        'Click',
      ],
      entry_points={
        'console_scripts': [
            'spag = spag:cli'
        ]
      },
      zip_safe=False)
