# Security policy

## Supported versions

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a vulnerability

Please report privately through GitHub's "Report a vulnerability" button on
this repository's Security tab (private vulnerability reporting). Don't open a
public issue for a security problem.

## Scope

poe2-core treats all of its input as untrusted:

- Client.txt lines (the client log);
- launcher files such as Steam library lists and app manifests;
- symlinks;
- Wine and Proton prefixes.

The library and the `poe2` CLI only read. They never write files and never use
the network. These count as bugs, and we want to hear about them:

- a panic, unbounded memory use or a hang on any input;
- reading outside the documented locations listed in the README's
  [Where it looks](README.md#where-it-looks) section.

When you report, never attach a real Client.txt; anonymise it as
CONTRIBUTING.md describes.
