# Comprehensive Guide

If you are reading this guide, that means you are curious about how OasysDB is designed, why it is designed that way, and how to use it for your project.

Thank you and welcome 🤗

My biggest goal for OasysDB is to make it **boring**. Not boring in a bad way, but, boring in a way to is predictable, easy to use, with no surprises. I want to make it so that you can use and rely on OasysDB without having to worry about it.

For that, I made some quite opinionated design decisions that I believe will help OasysDB achieve that goal. In this guide, I will explain those decisions and how they affect the usage of OasysDB.

### Table of Contents

- [Inner Workings](#inner-workings)
  - [Vector Record](#vector-record)
  - [Vector ID: Auto Incremented](#vector-id-auto-incremented)
  - [Persistence to Disk](#persistence-to-disk)
    - [Notes & Tips](#notes--tips)
- [Indexing Algorithm](#indexing-algorithm)
  - [Intro to HNSW](#intro-to-hnsw)
  - [Index Configuration](#index-configuration)
  - [Distance Metric](#distance-metric)
    - [Cosine VS Normalized Cosine](#cosine-vs-normalized-cosine)
  - [Relevancy Score](#relevancy-score)
- [Conclusion](#conclusion)
  - [Relevant Resources](#relevant-resources)

# Inner Workings

You can think of OasysDB as a NoSQL database optimized for vector operations because of how the data is indexed. Instead of using a traditional B-Tree or LSM-Tree, OasysDB uses [HNSW](#indexing-algorithm) algorithm to index the data in the form of graphs.

Besides that, OasysDB shares similar concept with traditional NoSQL databases. It stores data in collections, where each collection contains multiple records.

## Vector Record

When you want to store a vector in OasysDB, you will insert vector record objects. This object contains the vector itself and some metadata. The metadata object can be used to store any information you need to associate with the vector.

**Metadata types:**

- Text
- Number
- Array
- Object

## Vector ID: Auto Incremented

When you insert a vector record, OasysDB will automatically assign an integer ID to the record that is auto-incremented with every inserts. This ID is unique within the collection and will be used to reference the vector record.

I made this decision to make the indexing algorithm more efficient and performant. Compared to UUID which is 128-bit or string ID which can be any length, an integer U32 ID is only 32-bit. This means that the indexing algorithm can work with smaller and more predictable data size.

**The 2 downsides of this decision are:**

- You cannot specify the ID when inserting a vector record.
- A collection is limited to store around 4 billion records.

## Persistence to Disk

By default, due to the nature of the vector indexing algorithm, OasysDB stores the vector record data in memory via the collection interface. This means that unless persisted to disk via the database save collection method, the data will be lost when the program is closed.

Under the hood, OasysDB serializes the collection using [Serde](https://github.com/serde-rs/serde) and saves it to the database file using [Sled](https://github.com/spacejam/sled). Because of this, **whenever you modify a collection, you need to save the collection back to the database to persist the changes to disk.**

When opening the database, OasysDB doesn't automatically load the collections from the database file into memory as this would be inefficient if you have many collections you don't necessarily use all the time. Instead, you need to load the collections you want to use into memory manually using the get collection method.

### Notes & Tips

The serialization and the deserialization process is compute-intensive and can be rather slow. This is why to optimize the performance of your application, you should follow these tips:

- Save the collection to disk only when you're totally done with modifying it.
- Load only the collection you need into the memory as they could take up a good chunk of memory.
- If you use a collection for multiple processes, consider keeping it in-memory as a global state to avoid reloading it.

If you have any questions or need help with optimizing the performance of your application, feel free to ask me on the [Discord](https://discord.gg/bDhQrkqNP4) server.

I'm always happy to help you out 🤗

# Indexing Algorithm

This is arguably the most important part of OasysDB ⭐️

The indexing algorithm is what makes OasysDB a vector database and what allows you to perform fast similarity searches on your vectors records.

OasysDB uses the HNSW (Hierarchical Navigable Small World) algorithm. We're not going to dive deep into the algorithm in this guide, but, I will explain how it works in the context of OasysDB.

## Intro to HNSW

HNSW is a graph-based indexing algorithm. It consists of multiple layers containing nodes referencing other nodes (neighbors). These nodes represent the vector IDs of the records in the collection.

When you insert vector records into a collection, OasysDB will:

1. Generate vector IDs for the records.
2. Calculate distances between the new and existing vectors.
3. Place nodes and cluster them based on their similarity in the layers.
4. Store the other data in HashMaps for fast access.

Because OasysDB stores the vector IDs in the index graph as nodes, having an auto-incremented integer as the vector ID is important for memory efficiency and performance.

## Index Configuration

OasysDB allows you to configure the index parameters when creating a collection. As of the current version, these configurations can't be changed after the collection is created. These configurations include:

- **M**: The maximum number of neighbor connections to keep for each node when building the index or inserting a new vector record. OasysDB uses M of 32 by default and this value works well for most use cases. As of the current version, you can't change this value at all.

- **EF Construction**: This parameter along with the M parameter determines how well the index will be constructed. The higher the EF Construction value, the slower the index construction will be, but, the more accurate the index will be up to a certain point.

  According to [HNSWLIB's documentation](https://github.com/nmslib/hnswlib/blob/master/ALGO_PARAMS.md), to check if EF Construction value is good enough is to measure the recall for the search operation with k = M and EF Search = EF Construction. If the recall is lower than 0.9, than there is room for improvement.

- **EF Search**: This parameter determines how many nodes to visit when searching for the nearest neighbors. The higher the EF Search value, the more accurate the search result will be, but, the slower the search will be.

  EF Search value should be set to a value higher than k (the number of neighbors you want to find) when performing a search operation.

- **ML**: This parameter determines how likely it is for a node to be placed in the higher layer. This multiplier is what allows HNSW to be the most dense at the bottom and the least dense at the top keeping the search operation efficient. The optimal value for ML is 1 / ln(M). In OasysDB, this would be around 0.2885.

OasysDB has more parameters that you can configure but not directly related to the index configuration. For those parameters, we will discuss it in the next section 😁

## Distance Metric

For collections in OasysDB, you can specify the distance metric to use when calculating the distance between vectors. The distance metric is used mostly when inserting a new vector record into the collection and a bit when performing a search operation.

As of the current version, OasysDB supports the following distance metrics:

- [Euclidean](https://en.wikipedia.org/wiki/Euclidean_distance)
- [Cosine Similarity](https://en.wikipedia.org/wiki/Cosine_similarity)
- [Dot Product](https://en.wikipedia.org/wiki/Dot_product)

### Cosine VS Normalized Cosine

A lot of embedding models seem to recommend using cosine similarity as the distance metric. This is because cosine similarity is invariant to the magnitude of the vectors and only measures the angle between them which makes it easy to work with.

With OasysDB, you can use the default **Cosine** or the **Normalized Cosine** distance metric.

With the default Cosine, it will calculate both the dot product and the magnitude of the vectors to calculate the cosine similarity as you would expect.

But for models that already normalize the vectors, you can use the Normalized Cosine distance metric. This will skip the magnitude calculation and only calculate the dot product which will result in a faster insert and search operation.

## Relevancy Score

Relevancy score is a big part of OasysDB. It allows you to essentially exclude vectors that are not relevant to your search query.

Unlike other configurations, the relevancy score can be changed after the collection is created. I even encourage you to experiment with different relevancy scores to see what works best for your use case 😁

Relevancy score is a float value and it functions differently based on the distance metric you use.

For example, for the Cosine distance metric, since the cosine similarity value ranges from -1 to 1, where 1 would be the most similar, if you set the relevancy score to 0.8, OasysDB will only return vectors that have a cosine similarity of 0.8 or higher.

Another example, for the Euclidean distance metric, since the Euclidean distance value ranges from 0 to infinity, where 0 would be the most similar, if you set the relevancy score to 0.2, OasysDB will only return vectors that have an Euclidean distance of 0.2 or lower.

This is why it's important to experiment with different relevancy scores to see what works best for your use case.

# Conclusion

In short, use OasysDB to keep your sanity 😂

I hope this guide has given you a good understanding of how OasysDB works and how to use it for your project. If you have any questions or need help with anything Rust related, join the [Discord](https://discord.gg/bDhQrkqNP4) server and share them with me.

~ Edwin

### Relevant Resources

- [HNSW by Pinecone](https://www.pinecone.io/learn/series/faiss/hnsw/)
- [HNSW Algorithm by Lantern](https://lantern.dev/blog/hnsw)
- [What Are Vector Embeddings?](https://www.analyticsvidhya.com/blog/2020/08/information-retrieval-using-embeddings/)
- [OpenAI Embeddings](https://platform.openai.com/docs/guides/embeddings/frequently-asked-questions)
