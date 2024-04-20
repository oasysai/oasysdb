# Comprehensive Guide

If you are reading this guide, that means you are curious about how OasysDB is designed, why it is designed that way, and how to use it for your project.

Thank you and welcome 🤗

My biggest goal for OasysDB is to make it **boring**. Not boring in a bad way, but, boring in a way to is predictable, easy to use, with no surprises. I want to make it so that you can use and rely on OasysDB without having to worry about it.

For that, I made some quite opinionated design decisions that I believe will help OasysDB achieve that goal. In this guide, I will explain those decisions and how they affect the usage of OasysDB.

### Table of Contents

# Inner Workings

You can think of OasysDB as a NoSQL database optimized for vector operations because of how the data is indexed. Instead of using a traditional B-Tree or LSM-Tree, OasysDB uses [HNSW](#index-algorithm) algorithm to index the data in the form of graphs.

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

# Indexing Algorithm

sled
hnsw
vector id
metadata
