# jsongrep

Grep json.

## Usage

`jsongrep -h`

### Example

```shell
% (echo '{"s":"Sirius","i":0}';echo 'not json';echo '{"a":[]}';echo '{"s":1}';echo '{"s":"xirius"}';echo '{"s":"sirius"}') | jsongrep -r '{"query":{"type":"raw","pair":{"p":"/s","cond":{"type":"match","mtype":"regex","value":{"type":"string","value":"[sS]irius"}}}}}'
{"s":"Sirius","i":0}
line 2: expected ident at line 1 column 2
line 3: Invalid pointer (pointer: "/s", value: "{\"a\":[]}")
line 4: Matcher type mismatch (matcher_type "Regex", matcher_value "String([sS]irius)", target "Int(1)", by "&jsongrep::query::Condition")
{"s":"sirius"}
```

## Development

1. `cargo install --force cargo-make`
2. `cargo make dev`
