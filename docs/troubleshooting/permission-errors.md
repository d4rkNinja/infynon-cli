# Permission Errors

Use this page when INFYNON cannot write files, install packages, or launch tools because of permissions.

## Check folder permissions

Make sure your user can write to:

```txt
~/.infynon/
<your project>/.infynon/
```

On Windows:

```txt
C:\Users\<user>\.infynon\
```

## npm global permission

If npm global install fails, fix npm permissions or use a Node version manager.

```bash
npm install -g infynon
```

## Package manager permissions

If a package install fails, try running the package manager directly to confirm whether the issue is INFYNON or the package manager:

```bash
npm install axios
pip install requests
cargo add serde
```

## Notes

Avoid running INFYNON as administrator unless you understand the effect on generated files and package installs.

## Related docs

- [Generated files](../reference/generated-files.md)
- [Installation issues](./installation-issues.md)
