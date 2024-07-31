# Database

The Database is the primary interface for interacting with OasysDB. It is
responsible for managing the connection to the SQL database and vector indices.

These are the most notable operations that can be performed with the Database:

- Creating a new index.
- Refreshing an existing index.
- Searching for vectors in an index.
- Deleting an index.

## Create Index

This method creates a new index in the database. The initial data for the index
will be loaded from the SQL table defined in the source configuration parameter.

### Parameters

- **name**: Name of the new index.
- **algorithm**: Vector indexing algorithm to use in the index.
- **source**: Source configuration for the index.

### Custom Index Parameters

When specifying the indexing algorithm, we can also pass custom parameters
specific to the algorithm. For example, if we are using the IVFPQ algorithm, we
can configure the number of centroids and the number of sub-quantizers like:

```json
{
  "centroids": 512,
  "max_iterations": 100,
  "sub_centroids": 256,
  ...
}
```

For more information about the available parameters for each algorithm, please
refer to each algorithm's documentation.

### Source Configuration

The source configuration defines how the data will be loaded from the SQL
database to create and refresh the index. For example, if we store our vectors
in a table called _embeddings_ and in a column called _vector_, we can define
the source configuration like:

```json
{
  "table": "embeddings",
  "primary_key": "id",
  "vector": "vector"
  ...
}
```

!!! danger "Primary Key Requirement"

    The primary key must be unique and not null with auto-incrementing integer
    as its type. This allows OasysDB to incrementally load the data from the
    table when refreshing the index.

!!! danger "Vector Column Requirement"

    The vector must be stored in either JSON (Recommended) or blob column data
    type. Without this, OasysDB won't be able to load the vectors from the
    source table.

### Source Metadata (Optional)

In OasysDB, we can also store metadata along with the vectors directly in the
index which is very useful to eliminate post-search queries to the SQL database.
For example, if we have the following table in SQLite:

```sql
CREATE TABLE articles (
  id INTEGER PRIMARY KEY,
  vector JSON NOT NULL,
  content TEXT NOT NULL
);
```

We can define the source configuration to store the content in the index:

```json
{
  "table": "articles",
  "primary_key": "id",
  "vector": "vector",
  "metadata": ["content"]
  ...
}
```

When we search the index later on, the metadata will be included in the search
results allowing us to use the data right away without querying our SQL database
for the metadata.

!!! info "Metadata Limitation"

    The metadata is limited to primitive data types like integer, float, string,
    and boolean. It's also worth noting that the number and size of the metadata
    will affect the overall memory usage of the index.

    Don't overuse it 😁

### Source Filter (Optional)

In the source configuration, we can also define an optional SQL filter to load
only a subset of our data for the index. This filtering will also apply when
refreshing the index incrementally.

Let's say that we have a SQLite table with the schema below:

```sql
CREATE TABLE articles (
  id INTEGER PRIMARY KEY,
  vector JSON NOT NULL,
  content TEXT,
  year INTEGER,
);
```

We can add a SQL filter to only load the articles from the year 2021:

```json
{
  "table": "articles",
  "primary_key": "id",
  "vector": "vector",
  "filter": "year = 2021" // Exclude WHERE keyword
  ...
}
```

!!! warning "SQL Injection Risk"

    Be careful not to use user input directly in the SQL filter as this can
    lead to SQL injection attacks. Always sanitize the input before using it
    in the filter.

## Refresh Index

This method updates an existing index with the latest data from the SQL table.
Under the hood, OasysDB will query the source table from the last primary key
inserted and insert the new data to the index incrementally.

Incremental insertion is very crucial here because it allows us to insert an
individual record to the index without rebuilding the entire index which can be
very slow.

### Parameters

- **name**: Name of the index to refresh.

!!! tip "Asynchronous Refresh"

    The refresh operation can be performed asynchronously. This allows us to
    refresh the index in the background and/or periodically without blocking
    the main thread.

## Search Index

This method performs a nearest neighbor search in the index and returns _K_
search results based on the query vector. The search results will include the
primary key, distance between the query vector and the result vector, and
optional metadata if defined in the source configuration.

In JSON format, the search results will look like:

```json
[
  {
    "id": 1,
    "distance": 0.123,
    "data": {
      "content": "OasysDB is awesome!"
    }
  },
  ...
]
```

### Parameters

- **name**: Name of the index to search.
- **query**: Query vector for the nearest neighbor search.
- **k**: Number of results to return.
- **filters**: Optional SQL-like filter to apply to the search results.

### Post-filtering (Optional)

When searching the index, we can additionally apply post-filtering to the search
operation against the metadata stored in the index.

Let's say that we have the following setup for our index:

=== "SQLite Table"

    ```sql
    CREATE TABLE articles (
      id INTEGER PRIMARY KEY,
      vector JSON NOT NULL,
      year INTEGER
    );
    ```

=== "Source Configuration"

    ```json
    {
      "table": "articles",
      "primary_key": "id",
      "vector": "vector",
      "metadata": ["year"]
    }
    ```

Since we have the year metadata stored in the index, we can apply post-filtering
to the search operation by adding a filter string to the filters parameter:

```json
{
  "name": "index",
  "query": [0.1, 0.2, 0.3, ...],
  "k": 10,
  "filters": "year = 2021" // SQL-like filtering
}
```

This operation will only return the search results where the year metadata is
equal to 2021. There are also other operators we can use for the filtering and
these are the supported operators with their compatible metadata types:

| Operator | Description           | Metadata Type  |
| -------- | --------------------- | -------------- |
| =        | Equal                 | All            |
| !=       | Not Equal             | All            |
| <        | Less Than             | Integer, Float |
| <=       | Less Than or Equal    | Integer, Float |
| >        | Greater Than          | Integer, Float |
| >=       | Greater Than or Equal | Integer, Float |
| CONTAINS | Contains              | String         |

These operators can also be combined with the **AND** or **OR** logical
operators to create more complex filtering conditions. However, we can only use
one type of join operator at a time. For example:

```json
{
  ...
  "filters": "year >= 2020 AND year <= 2022"
}
```

!!! note "Filtering Limitation"

    The filtering is limited to the metadata stored in the index. If we add a
    filter with a column that is not included in the metadata, the search
    operation will return an empty result since none of the metadata matches
    the filter.

## Delete Index

This method deletes an existing index from the database and automatically
releases the index from the indices pool if it's loaded.

Since by default, the index is persisted on disk, deleting the index will also
remove the index file from the disk. This operation is useful when we want to
free up the disk space by removing indices that are no longer needed.

**This operation is irreversible!**

### Parameters

- **name**: Name of the index to delete.

## Indices Pool

The Database also contains indices pool to manage multiple indices in-memory.
This is useful when we have multiple indices we frequently use allowing us to
avoid the overhead of loading the index from disk which can be slow.

By default, performing any operation related to an index like search or refresh
will load the index to the pool. If we want to release the index from the pool,
we can use the `release_indices` method.
