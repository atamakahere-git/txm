<div align="center">
  <h1>TXM.py</h1>
  <p>Python bindings for TXM.</p>
</div>

### Example

```
pip install txm-py
```

```python
import txm

print(txm.render("E=mc^2"))
```

### Build

```
uv venv
source .venv/bin/activate
uv pip install maturin
uv run maturin develop
```

And then in a python repl:

```python
import txm
# do stuff
```

## License
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
