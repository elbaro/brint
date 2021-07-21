#!/usr/bin/env python

from distutils.core import setup

setup(
    name='brint',
    version='0.1.0',
    author='elbaro',
    # author_email='elbaro@users.noreply.github.com',
    url='https://github.com/elbaro/brint/',
    description='A feature-gate library using semver',
    packages=['brint'],
    classifiers=[
        'Topic :: Software Development :: Documentation',
        'Topic :: Software Development :: Bug Tracking',
        'Topic :: Software Development :: Version Control',
    ],
    install_requires=[
        'semver',
        'rtoml',
    ],
    keywords=[],
    license='MIT',
)
