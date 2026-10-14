//! ConectaStore: sistema de recomendação de produtos baseado em grafos para a
//! MegaStore.
//!
//! Módulos:
//! - `modelo`: entidades (Produto, Cliente, Categoria), vértices, arestas e erros.
//! - `avaliacao`: notas de 1 a 5 e sua conversão em peso de aresta.
//! - `grafo`: grafo ponderado em lista de adjacência e BFS.
//! - `catalogo`: cadastro e consulta, mantendo o grafo atualizado.
//! - `recomendacao`: algoritmo de recomendação (BFS com profundidade limitada).
//! - `dados`: catálogo de exemplo e gerador de dados sintéticos.
//! - `medicao`: medição de desempenho com volumes crescentes.

pub mod avaliacao;
pub mod catalogo;
pub mod dados;
pub mod grafo;
pub mod medicao;
pub mod modelo;
pub mod recomendacao;
