# `latch-scheme`

HTTP latch that makes decisions based on the request's scheme.

Authorization decisions are based on the wasi:config with the lowercase scheme value as the config key, using `denied` or `abstained` as the value. A default decision may specified under the `*` key.

## The `latch-scheme` World

- imports `wasi:config/store@0.2`
- exports `componentized:http/latch`
