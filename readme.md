![Oasys](/assets/banner.png)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](/docs/code_of_conduct.md)
[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)

## Getting started

### With Docker

The easiest way to get started is to use Docker. You can run the following command to start the server:

```bash
docker pull ghcr.io/oasysai/oasysdb:latest
```

This will pull the latest version of the server from the GitHub Container Registry. You can then run the server with the following command:

```bash
docker run \
    --platform linux/amd64 \
    --publish 3141:3141 \
    --env OASYSDB_DIMENSION=512 \
    --env OASYSDB_TOKEN=token \
    ghcr.io/oasysai/oasysdb:latest
```

- `OASYSDB_DIMENSION`: An integer representing the dimension of your embedding. Different embedding model will have different dimension. For example, OpenAI Ada 2 has a dimension of 1536.
- `OASYSDB_TOKEN`: A string that you will use to authenticate with the server. You need to add `x-oasysdb-token` header to your request with the value of this environment variable.

This will start OasysDB that is accessible on port `3141`. You can change this by changing the port number in the `--publish` flag and setting the `OASYSDB_PORT` environment variable to the port number that you want to use.

### Testing the server

You can test the server by calling `GET /` using your favorite HTTP client. For example, you can use `curl`:

```bash
curl http://localhost:3141
```

You can replace `localhost` with the IP address of the server if you are running the server on a remote machine.

## Usage

At its core, OasysDB has 2 main components: key-value store and embedding index. These are the documentations of OasysDB core functionalities which should be enough to get you started. But, if you need a more thorough documentation about OasysDB, please refer to [OasysDB documentation](https://docs.oasysai.com).

### Set a value

Create or update the value of a key. See below for the expected format of the request body.

```
POST /values
```

```json
{
  "key": "string",
  "value": {
    "embedding": [0.0, 0.0],
    "data": {}
  }
}
```

The `embedding` field is a list of floating-point numbers with the dimension specified by the `OASYSDB_DIMENSION` environment variable.

The `data` field is an object that can be used to store additional information about the embedding. For example, texts and their sources. Currently, this only support string keys and values. This field is optional but highly recommended. Otherwise, querying the index will only return empty objects.

### Build the index

Build the HNSW index. This operation is required before you can query the index. We use HNSW as the underlying algorithm for the embedding index and for that, we use [instant-distance](https://github.com/instant-labs/instant-distance) crate.

```
POST /index
```

Optionally, you can specify `ef_construction` and `ef_search` in the request body. These are the parameters for the HNSW algorithm. By default, we use `100` for both parameters.

```json
{
  "ef_construction": 100,
  "ef_search": 100
}
```

### Query the index

Query the index given an embedding. See below for the expected format of the request body.

```
POST /index/query
```

```json
{
  "embedding": [0.0, 0.0],
  "count": 10
}
```

The dimension of `embedding` must match the dimension specified by the `OASYSDB_DIMENSION` environment variable.

This will return a list of value's data that are associated with the nearest neighbors of the given embedding. The length of the list is specified by the `count` field.

If, for example, your value's data contains `text` and `source` information, this is an example of the response:

```json
[
  {
    "text": "string",
    "source": "string"
  }
]
```

## Disclaimer

This project is still in the early stages of development. We are actively working on it and we expect the API and functionality to change. We do not recommend using this in production yet.

We also don't have a benchmark yet. We are working on it and we will publish the results once we have them.

## Contributing

We welcome contributions from the community. Please see [contributing.md](/docs/contributing.md) for more information.

We are also looking for advisors to help guide the project direction and roadmap. If you are interested, please contact us via [Discord](https://discord.gg/bDhQrkqNP4) or alternatively, you can email me at edwin@oasysai.com.

## Code of Conduct

We are committed to creating a welcoming community. Any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).
