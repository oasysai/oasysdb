![Oasys](/assets/banner.png)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg?style=for-the-badge)](https://opensource.org/licenses/Apache-2.0)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](/docs/code_of_conduct.md)
[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)

## What is OasysDB?

OasysDB is a vector database that can be used to store and query high-dimensional vectors. Our goal is to make OasysDB fast and easy to use. We are also working on making it easy to deploy and scale.

### Features

- **HTTP-based API**: All operations are exposed via a RESTful API. This makes it easy to integrate with other systems without having to install any client libraries.

- **Persistent storage**: Embeddings and graphs data are stored on disk and are persisted across restarts.

- **HNSW indexing**: OasysDB uses the HNSW algorithm to build graphs to index embeddings. This allows for fast and accurate nearest neighbor search.

- **Multi-graph support**: OasysDB supports multiple HNSW graphs. This allows you to version and customize your graphs to suit different use cases. For example, optimizing speed for a specific query type or optimizing accuracy for a specific dataset.

## Getting Started

### Installation

The easiest way to get started is to use Docker. You can pull the latest image from GitHub Container Registry:

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

## Quickstart

To put it simply, these are the primary steps to get started with OasysDB: Setting values, creating a graph, and querying the graph.

### Setting a value

```
POST /values/<key>
```

```json
{
  "embedding": [0.1, 0.2, 0.3],
  "data": {
    "type": "fact",
    "text": "OasysDB is awesome!"
  }
}
```

This endpoint sets a value for a given key. The value is an embedding and an optional data object. The embedding is a list of floating-point numbers. The data object is a JSON object of string keys and values.

### Creating a graph

```
POST /graphs
```

Optional request body:

```json
{
  "name": "my-graph",
  "ef_construction": 10,
  "ef_search": 10,
  "filter": {
    "type": "fact"
  }
}
```

This endpoint creates a graph. The graph is used to query for nearest neighbors. If there is no data provided, the server will create a default graph with the name `default` and the default `ef_construction` and `ef_search` values of 100 for both.

The filter object is used to filter the values that are added to the graph. For example, if you only want to add values with the `type` data key set to `fact`, you can use the filter object above.

The filter operation is similar to the `AND` operation. This means if you have multiple filters, the server will only add values that match all of filters. Without a filter, the server will add all values to build the graph.

### Querying the graph

```
POST /graphs/<name>/query
```

```json
{
  "embedding": [0.1, 0.2, 0.3],
  "k": 10
}
```

This endpoint queries the graph for the nearest neighbors of the given embedding. The `k` parameter is the number of nearest neighbors to return.

### Note

- All embedding dimensions must match the dimension configured in the server using the `OASYSDB_DIMENSION` environment variable.
- Requests to `/graphs` and `/values` endpoints must include the `x-oasysdb-token` header with the value of the `OASYSDB_TOKEN` environment variable.
- OasysDB doesn't support automatic graph building due to its versioning and filtering nature. This means whenever you add or remove a value, you need to rebuild the graph.

### More resources

These are the primary steps to get started with OasysDB. If you want to go deeper, we have more resources available on our documentation site: [OasysDB Docs](https://www.oasysai.com/docs).

If you have any questions, you can join our [Discord](https://discord.gg/bDhQrkqNP4) and ask us there or open a discussion on GitHub for your question. We'll be happy to help.

## Disclaimer

This project is still in the early stages of development. We are actively working on it and we expect the API and functionality to change. We do not recommend using this in production yet.

We also don't have a benchmark yet. We are working on it and we will publish the results once we have them.

## Contributing

We welcome contributions from the community. Please see [contributing.md](/docs/contributing.md) for more information.

We are also looking for advisors to help guide the project direction and roadmap. If you are interested, please contact us via [Discord](https://discord.gg/bDhQrkqNP4) or alternatively, you can email me at edwin@oasysai.com.

## Code of Conduct

We are committed to creating a welcoming community. Any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).
