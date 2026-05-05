# Installation Issues

Use this page when `infynon` is not found, the version is wrong, or the npm wrapper cannot run the native binary.

## Check version

```bash
infynon --version
```

## Check npm install

```bash
npm list -g infynon
npm root -g
```

## Reinstall

```bash
npm uninstall -g infynon
npm install -g infynon
```

## Check PATH

If `infynon` is not found, make sure your npm global binary directory is on `PATH`.

## Notes

Use Node.js 18 or newer.

## Related docs

- [Installation](../installation.md)
- [Windows issues](./windows-issues.md)
