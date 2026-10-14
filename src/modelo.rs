//! Entidades do domínio da MegaStore e os tipos que compõem o grafo.

use std::fmt;

/// Identificador numérico de clientes, produtos e categorias.
///
/// `u32` ocupa 4 bytes, é copiado sem custo (`Copy`) e comporta mais de 4 bilhões
/// de registros, o que basta para um catálogo de milhões de produtos.
pub type Id = u32;

#[derive(Debug, Clone, PartialEq)]
pub struct Categoria {
    pub id: Id,
    pub nome: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Produto {
    pub id: Id,
    pub nome: String,
    pub categoria_id: Id,
    pub preco: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cliente {
    pub id: Id,
    pub nome: String,
}

/// Vértice do grafo. Clientes, produtos e categorias são vértices de tipos
/// diferentes; o tipo faz parte da chave, então o cliente 1 e o produto 1 são
/// vértices distintos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Vertice {
    Cliente(Id),
    Produto(Id),
    Categoria(Id),
}

impl fmt::Display for Vertice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Vertice::Cliente(id) => write!(f, "cliente {id}"),
            Vertice::Produto(id) => write!(f, "produto {id}"),
            Vertice::Categoria(id) => write!(f, "categoria {id}"),
        }
    }
}

/// Aresta da lista de adjacência: guarda só o destino e o peso, porque a origem
/// já é a chave sob a qual a aresta está armazenada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aresta {
    pub destino: Vertice,
    pub peso: f64,
}

/// Erros de domínio do catálogo. Substituem as exceções do Python: quem chama
/// é obrigado pelo compilador a tratar cada caso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErroCatalogo {
    ClienteNaoEncontrado(Id),
    ProdutoNaoEncontrado(Id),
    CategoriaNaoEncontrada(Id),
    IdDuplicado(Vertice),
    NotaInvalida(u8),
}

impl fmt::Display for ErroCatalogo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroCatalogo::ClienteNaoEncontrado(id) => write!(f, "cliente {id} não encontrado"),
            ErroCatalogo::ProdutoNaoEncontrado(id) => write!(f, "produto {id} não encontrado"),
            ErroCatalogo::CategoriaNaoEncontrada(id) => {
                write!(f, "categoria {id} não encontrada")
            }
            ErroCatalogo::IdDuplicado(v) => write!(f, "{v} já cadastrado"),
            ErroCatalogo::NotaInvalida(n) => write!(f, "nota {n} inválida (use 1 a 5)"),
        }
    }
}

impl std::error::Error for ErroCatalogo {}
