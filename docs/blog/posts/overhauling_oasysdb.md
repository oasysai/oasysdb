---
date: 2024-06-22

authors:
  - edwinkys

categories:
  - Log
---

# DevLog #1: OasysDB Overhaul

OasysDB is a project that I started in January of this year, and honestly, it has been an incredible learning experience. With it, I've gained quite extensive experience in databases, machine learning algorithms, and low-level programming concepts. But, with this knowledge, I realize that the current design of OasysDB is not enough for production use.

<!-- more -->

After careful consideration, I've decided to rewrite OasysDB from the ground up. The new version will be designed to incorporate all the essential features needed for a production-ready vector database system.

This includes, but is not limited to:

- Transitioning from an embedded to a client-server model for better scalability and isolation.
- Designing an efficient storage engine tailored for analytical production workloads.
- Implementing concurrent query processing to improve throughput and reduce latency.
- Utilizing advanced vector indexing algorithms for enhanced recall performance, especially in hybrid search scenarios.
- Incorporating an industry-standard query planner and optimizer to enhance query performance.
- Enhancing documentation and testing to ensure the system's robustness and reliability.

Here's a high-level overview of the new architecture:

![OasysDB Architecture](https://i.postimg.cc/QdVVSs3M/Infrastructure.png)

## Progress Update

Today, I started working on the new version of OasysDB. I've established the project structure, implemented the foundational data structures for the collection and storage engine, and set up the initial framework for client-server communication.

I will be posting regular updates (once or twice a week) on my progress, which may include in-depth explorations of the system's technical aspects. If you want to follow along with the development process, you can find the project on GitHub: [OasysDB](https://github.com/oasysai/oasysdb).

## Conclusion

I'm really excited about the potential of the new OasysDB and the challenges that lie ahead. I believe this overhaul will lead to a robust and scalable vector database system perfect for a wide range of AI applications.

If you're into databases and AI, I encourage you to follow along with the development process as I share my insights, challenges, and victories in this DevLog series. If you have experience in this field, your feedback and suggestions would be greatly appreciated.
