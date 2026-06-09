Please create a new release using the forgejo CLI (`fj`). Make sure it is in the
following format:

```md
# Version <RELEASE VERSION HERE>

## Release notes

- ... (describe changes/fixes/etc)
- ... (describe changes/fixes/etc)
```

> [!NOTE]
>
> - DO NOT include notes about non-app related changes in the release notes.

Make sure to create a corresponding tag for the release using `fj tag create
v<RELEASE VERSION HERE>`.

Once the forgejo release and tag are created. Please make sure to mirror only
the release on GitHub (do not push any code or tags, these are automatically
mirrored by Codeberg). There may be a delay before the tag is mirrored on to
GitHub, so please be patient. If the tag does not appear for 10 minutes, please
continue to the next step but inform me at the end.

Once all steps are complete, please finally make sure to bump the version in the
`Cargo.toml` file (ask me to specify whether it's a patch, minor, or major
release), and commit the change using the conventional commit format.
