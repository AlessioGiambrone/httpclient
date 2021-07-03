<div align="center">
  <h1 align="center" style="font-family:'Lucida Console', monospace">HTTPCLIENT</h1>
</div>
<div align="center">
 <strong>
  HTTP calls for everyone
 </strong>
</div>

</br>
</br>

A CLI for parsing and executing requests from `.http` files analogous to 
[JetBrains](https://www.jetbrains.com/help/idea/http-client-in-product-code-editor.html)' 
and 
[VisualStudio](https://github.com/Huachao/vscode-restclient)'s GUI ones.

## Usage

Executing an `example.http` file with content

```http
https://api.wheretheiss.at/v1/satellites
```

will show the raw response:

```bash
% httpclient example.http
[{"name":"iss","id":25544}]
```

This response can be useful when using `httpclient` piped with other programs.

In case of a JSON response you can use `httpclient` together with 
[`jq`](https://stedolan.github.io/jq/) so that you'll have a well-formatted output:

```bash
% httpclient example.http | jq .
[
  {
    "name": "iss",
    "id": 25544
  }
]
```

or use the `-v` flag, that will also print some additional information 
in a human-readable form:

```bash
% httpclient -v example.http
200 OK - 790.38613ms
cache-control: "max-age=0, no-cache"
content-length: "27"
content-type: "application/json"

[
  {
    "name": "iss",
    "id": 25544
  }
]
```

## `.HTTP` file synax

### TL;DR

```http
HTTPVERB https://your.address/?parameter1=inline
  &parameter2=next_line
header1=value1
header2=value2

{
  "json": "body"
}
```

### Comments and separators

Comments are identified with `//` or `#`.

Use `###` to delimitate different requests, that can be selected using the (0-indexed) `-n` parameter.

### URL parameters

URL parameters can be either put inline with the URL or one for each line after it,
padded to the right.

Examples (both valid):

```http
GET https://api.wheretheiss.at/v1/satellites/25544/positions
  ?timestamps=1609462861
  &units=kilometers
###
GET https://api.wheretheiss.at/v1/satellites/25544/positions?timestamps=1609462861&units=kilometers
```

### Headers

Headers must be set after the URL and its parameters, without spaces on the left.

Header name must be separated from its value by `: ` (colon followed by space).

Example:

```http
GET https://api.wheretheiss.at/v1/satellites/25544/positions
auth: something
```

### Payload

After headers leave a blank line; after that everything will be treated as payload.

Example:

```http
GET https://api.wheretheiss.at/v1/satellites/25544/positions

{
  "some": "payload"
}
```

### TODOs

- [ ] VSCode environment file support
- [ ] JetBrains environment file support
- [ ] CURL export
- [ ] CURL import
