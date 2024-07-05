# Terms

If you're new to RAG, vector search, and related concepts, this documentation
will guide you through the key terms and principles used in modern LLM-based
applications.

This documentation attempts to provide a very high-level overview of the key
concepts and terms used in the LLM ecosystem. For a more in-depth understanding,
we recommend reading other dedicated resources.

With that said, let's get started!

## Embedding

Embedding is a way to represent unstructured data as numbers to capture the
semantic meaning of the data. In the context of LLMs, embeddings are used to
represent words, sentences, or documents.

Let's say we have a couple of words that we want to represent as numbers. For
simplicity, we will only consider 2 aspects of the words: edibility and
affordability.

| Word   | Edibility | Affordability | Label        |
| ------ | --------- | ------------- | ------------ |
| Apple  | 0.9       | 0.8           | Fruit        |
| Apple  | 0.0       | 0.0           | Tech Company |
| Banana | 0.8       | 0.8           | ?            |

In the table above, we can roughly deduce that the first apple is a fruit, while
the second apple refers to a tech company. If we were to deduce if the banana
here is a fruit or a tech company we never heard about, we could roughly say
that it's a fruit since it has similar edibility and affordability values as the
first apple.

In practice, embeddings are much more complex and have many more dimensions,
often capturing various semantic properties beyond simple attributes like
edibility and affordability. For instance, embeddings in models like Word2Vec,
GloVe, BERT, or GPT-3 can have hundreds or thousands of dimensions. These
embeddings are learned by neural networks and are used in numerous applications,
such as search engines, recommendation systems, sentiment analysis, and machine
translation.

Moreover, modern LLMs use contextual embeddings, meaning the representation of a
word depends on the context in which it appears. This allows the model to
distinguish between different meanings of the same word based on its usage in a
sentence.

Note that embedding and vector are often used interchangeably in the context of
LLMs.

## Indexing

Indexing is the process of organizing and storing data to optimize search and
retrieval efficiency. In the context of RAG and vector search, indexing
organizes data based on their embeddings.

Let's consider 4 data points below with their respective embeddings representing
features: alive and edible.

| ID  | Embedding  | Data   |
| --- | ---------- | ------ |
| 1   | [0.0, 0.8] | Apple  |
| 2   | [0.0, 0.7] | Banana |
| 3   | [1.0, 0.4] | Dog    |
| 4   | [0.0, 0.0] | BMW    |

To illustrate simple indexing, let's use a simplified version of the NSW
(Navigable Small World) algorithm. This algorithm establishes links between data
points based on the distances between their embeddings:

```
1 -> 2, 3
2 -> 1, 3
3 -> 2, 4
4 -> 3, 2
```

## ANNS

ANNS is a technique for efficiently finding the nearest data points to a given
query, albeit approximately. While it may not always return the exact nearest
data points, ANNS provides results that are close enough. This probabilistic
approach balances accuracy with efficiency.

Let's take the index we have created in the previous section as an example.
Imagine we have a query with these specific constraints:

- Find the closest data to [0.0, 0.9].
- Calculate a maximum of 2 distances using the Euclidean distance formula.

Here's how we can utilize the index to find the closest data point based on this
constraint:

1. We start at a random data point, say 4, which is linked to 3 and 2.
2. We calculate the distances and find that 2 is closer to [0.0, 0.9] than 3.
3. We determine that the closest data to [0.0, 0.9] is Banana.

This method isn't perfect; in this case, the actual closest data point to [0.0,
0.9] is Apple. But, under these constraints, linear search would rely heavily on
chance to find the nearest data point. Indexing mitigates this issue by
efficiently narrowing down the search based on data embeddings.

In real-world applications with millions of data points, linear search becomes
impractical. Indexing, however, enables swift retrieval by structuring data
intelligently according to their embeddings.

Note that for managing billions of data points, sophisticated disk-based
indexing algorithms may be necessary to ensure efficient data handling.

## RAG

RAG (Retrieval-Augmented Generation) is a framework that combines information
retrieval and large language models (LLMs) to generate high-quality,
contextually relevant responses to user queries. This approach enhances the
capabilities of LLMs by incorporating relevant information retrieved from
external sources into the model's input.

In practice, RAG works by retrieving relevant information from a vector
database, which allows efficient searching for the most relevant data based on
the user query. This retrieved information is then inserted into the input
context of the language model, providing it with additional knowledge to
generate more accurate and informative responses.

Below is an example of a prompt with and without RAG in a simple Q&A scenario:

=== "Without RAG"

    ```text
    What is the name of my dog?
    ```

    > LLM: I don't know.

=== "With RAG"

    ```text
    Based on the context below:
    I have a dog named Pluto.

    Answer the following question: What is the name of my dog?
    ```

    > LLM: The name of your dog is Pluto.

By integrating retrieval with generation, RAG significantly improves the
performance of LLMs in tasks that require specific, up-to-date, or external
information, making it a powerful tool for various applications such as customer
support, knowledge management, and content generation.

## Token

A token is a unit of text that AI models use to process and understand natural
language. Tokens can be words, subwords, or characters, depending on the model's
architecture. Tokenization is a crucial preprocessing step in natural language
processing (NLP) and is essential for breaking down text into manageable pieces
that the model can process.

In this example, we'll use `WordPunctTokenizer` from the NLTK library to
tokenize the sentence: "OasysDB is awesome."

```py
from nltk.tokenize import WordPunctTokenizer

tokenizer = WordPunctTokenizer()
tokens = tokenizer.tokenize("OasysDB is awesome.")
print(tokens)
```

```py
["OasysDB", "is", "awesome", "."]
```

Tokenization plays a big role in LLMs and embedding models. Understanding
tokenization can help in various aspects, such as optimizing model performance
and managing costs.

Since many AI service providers charge based on the number of tokens processed.
So, you'll often encounter this term when working with LLMs and embedding
models, especially when determining the pricing of using a specific model.
