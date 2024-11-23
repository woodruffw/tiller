# tiller

Tiller tills TILs.

See <https://yossarian.net/til/> for an example of a Tiller-generated website,
and [woodruffw/til] for an example of Tiller's raw inputs.

## Usage

```bash
cargo install tiller
tiller --help
```

### Local development

The easiest way to iterate on a `tiller` managed website locally is
with `--dev` and a local HTTP server:

```bash
tiller --dev
python -m http.server -d site/ 1337
```

## License

MIT.

[woodruffw/til]: https://github.com/woodruffw/til

