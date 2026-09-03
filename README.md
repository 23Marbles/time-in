# TimeIn

Useful utilities for finding timings.

# Install

Install with:

```bash
cargo install timein
```

It will be installed as a binary called "time.*"

# Notes

The code structure is very messy and _may_ be changed.
This includes the cli flags _possibly_ being overhauled.
(The reason I say _may_ is because if I spend any more time working on a project
which finds how long it is until Sunday I might go insane)

# Modes

## In

Shows you what the time will be after a specified duration.

### Past

Shows you what the time will be a set duration past.

## Till

Shows you how long until a specified time.

## Since

An alias of `Till`
