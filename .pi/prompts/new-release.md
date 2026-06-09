Please create a new release using the forgejo CLI (`fj`). Make sure it is in the
following format:

```md
# Version <RELEASE VERSION HERE>

## Release notes

- ... (describe changes/fixes/etc)
- ... (describe changes/fixes/etc)
```

Make sure to create a corresponding tag for the release using `fj tag create
v<RELEASE VERSION HERE>`.

Please make sure to mirror the release and release notes on GitHub using the
`gh` CLI (don't push a tag).

Once all steps are complete, please finally make sure to bump the version in the
`Cargo.toml` file (ask me to specify whether it's a patch, minor, or major
release), and commit the change using the conventional commit format.
