# TXM
TXM (Terminal TeX Math) is a math rendering engine with LaTeX support.

# Example:
```
$ txm "f(x) = f(a) + f'(a)(x - a) + \frac{f''(a)}{2!}(x - a)^2 + \frac{f'''(a)}{3!}(x - a)^3+\dots"
┌───────────────────────────────────────────────────────────────────────┐
│                                                                       │
│                               f''(a)             f'''(a)              │
│ f(x) = f(a) + f'(a)(x - a) + ────────(x - a)² + ─────────(x - a)³ + ⋯ │
│                                 2!                 3!                 │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

```
$ txm "\frac{d}{dx}\quad\sin^{-1}(\frac{x}{a}) = \frac{1}{\sqrt{a^2 - x^2}}"
┌──────────────────────────────────┐
│                                  │
│  d      -1⎛  x  ⎞        1       │
│ ──── sin  ⎟ ─── ⎟ = ──────────── │
│  dx       ⎝  a  ⎠     ┌────────  │
│                      ╲│ a² - x²  │
│                                  │
└──────────────────────────────────┘
```

```
$ txm "\sqrt{\frac{(\frac{\alpha}{\beta})^{\frac{\gamma}{\delta}}}{(\sqrt{\gamma+\frac{\delta}{\lambda}})^{e^{i\pi}} + \binom{n}{k}}}"
┌────────────────────────────┐
│                            │
│  ┌──────────────────────── │
│  │              γ          │
│  │              ─          │
│  │              δ          │
│  │         ⎛ α ⎞           │
│  │         ⎟ ─ ⎟           │
│  │         ⎝ β ⎠           │
│  │ ─────────────────────── │
│  │              iπ         │
│  │             e           │
│  │ ⎛  ┌────── ⎞            │
│  │ ⎟  │     δ ⎟      ⎛ n ⎞ │
│  │ ⎟  │ γ + ─ ⎟    + ⎟   ⎟ │
│ ╲│ ⎝ ╲│     λ ⎠      ⎝ k ⎠ │
│                            │
└────────────────────────────┘
```

## License
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))
