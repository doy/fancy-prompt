# Changes

## 0.3.0

* Rerelease because apparently 0.2.0 and 0.2.1 were already released from
  somewhere?

## 0.2.0

* Handle USB power sources
* Don't assume that the default branch is always named master

## 0.1.5

* Strip off everything past a '.' in the hostname (OSX puts extra confusing
  data in the hostname that's not particularly useful)

## 0.1.4

* Fix shell prompt escaping (should fix input corruption in new zsh, probably
  among other things)

## 0.1.3

* Fix compilation and execution on macos (battery state is not yet supported
  though)

## 0.1.2

* Fix TERM=xterm-256color on recent Arch Linux
  (see https://github.com/Stebalien/term/issues/81)

## 0.1.1

* Fix `--version` output.

## 0.1.0

* Initial release.
