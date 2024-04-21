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
