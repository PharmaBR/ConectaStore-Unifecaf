//! Dados do sistema: um catálogo de exemplo pequeno e legível (para a
//! demonstração e os testes) e um gerador de catálogos sintéticos de qualquer
//! tamanho (para a medição de desempenho).

use crate::catalogo::Catalogo;
use crate::modelo::{ErroCatalogo, Id};

/// Catálogo de demonstração: 6 categorias, 18 produtos, 8 clientes.
pub fn catalogo_exemplo() -> Result<Catalogo, ErroCatalogo> {
    let mut c = Catalogo::new();

    let categorias = [
        (1, "Informática"),
        (2, "Livros"),
        (3, "Casa e Decoração"),
        (4, "Alimentos"),
        (5, "Vestuário"),
        (6, "Eletrônicos"),
    ];
    for (id, nome) in categorias {
        c.cadastrar_categoria(id, nome)?;
    }

    let produtos = [
        (10, "Notebook", 1, 4299.00),
        (11, "Mouse sem fio", 1, 89.90),
        (12, "Teclado mecânico", 1, 349.90),
        (13, "Monitor 27 polegadas", 1, 1299.00),
        (14, "Mousepad", 1, 39.90),
        (20, "Livro: Algoritmos", 2, 189.00),
        (21, "Livro: Programando em Rust", 2, 159.00),
        (22, "Livro: Estruturas de Dados", 2, 139.00),
        (30, "Luminária de mesa", 3, 119.00),
        (31, "Cadeira ergonômica", 3, 1590.00),
        (32, "Quadro decorativo", 3, 99.00),
        (40, "Café especial 500g", 4, 54.90),
        (41, "Chá verde", 4, 24.90),
        (50, "Camiseta dry-fit", 5, 69.90),
        (51, "Tênis de corrida", 5, 499.00),
        (60, "Fone Bluetooth", 6, 299.00),
        (61, "Smartwatch", 6, 899.00),
        (62, "Caixa de som portátil", 6, 349.00),
    ];
    for (id, nome, categoria, preco) in produtos {
        c.cadastrar_produto(id, nome, categoria, preco)?;
    }

    let clientes = [
        (1, "Ana"),
        (2, "Bia"),
        (3, "Caio"),
        (4, "Davi"),
        (5, "Eva"),
        (6, "Fábio"),
        (7, "Gil"),
        (8, "Helena"),
    ];
    for (id, nome) in clientes {
        c.cadastrar_cliente(id, nome)?;
    }

    // (cliente, produto, nota)
    let compras = [
        (1, 10, 5),
        (1, 11, 5),
        (1, 20, 4),
        (2, 11, 5),
        (2, 12, 5),
        (2, 31, 4),
        (2, 40, 5),
        (3, 12, 4),
        (3, 13, 5),
        (3, 60, 4),
        (4, 20, 5),
        (4, 21, 5),
        (4, 22, 4),
        (4, 40, 4),
        (5, 10, 4),
        (5, 31, 5),
        (5, 30, 5),
        (6, 51, 5),
        (6, 61, 5),
        (6, 60, 4),
        (6, 50, 3),
        (7, 40, 5),
        (7, 41, 4),
        (7, 32, 3),
        (8, 11, 4),
        (8, 12, 5),
        (8, 30, 4),
        (8, 21, 3),
    ];
    for (cliente, produto, nota) in compras {
        c.registrar_compra(cliente, produto, Some(nota))?;
    }

    Ok(c)
}

/// Gerador pseudoaleatório determinístico (xorshift64*). Sem dependências
/// externas e com a mesma sequência a cada execução, o que torna a medição
/// reproduzível.
pub struct Gerador {
    estado: u64,
}

impl Gerador {
    pub fn new(semente: u64) -> Self {
        Self {
            estado: semente.max(1),
        }
    }

    pub fn proximo(&mut self) -> u64 {
        let mut x = self.estado;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.estado = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Número em `0..limite`.
    pub fn ate(&mut self, limite: u64) -> u64 {
        self.proximo() % limite.max(1)
    }
}

/// Gera um catálogo sintético.
///
/// - `n_produtos / 100` categorias (no mínimo 1), com produtos distribuídos entre elas;
/// - cada cliente faz `compras_por_cliente` compras de produtos sorteados, com
///   nota de 1 a 5.
pub fn gerar_catalogo(
    n_clientes: u32,
    n_produtos: u32,
    compras_por_cliente: u32,
    semente: u64,
) -> Result<Catalogo, ErroCatalogo> {
    let mut g = Gerador::new(semente);
    let mut c = Catalogo::new();

    let n_categorias = (n_produtos / 100).max(1);
    for id in 0..n_categorias {
        c.cadastrar_categoria(id, &format!("Categoria {id}"))?;
    }
    for id in 0..n_produtos {
        let categoria = g.ate(n_categorias as u64) as Id;
        let preco = 10.0 + g.ate(5000) as f64;
        c.cadastrar_produto(id, &format!("Produto {id}"), categoria, preco)?;
    }
    for id in 0..n_clientes {
        c.cadastrar_cliente(id, &format!("Cliente {id}"))?;
        for _ in 0..compras_por_cliente {
            let produto = g.ate(n_produtos as u64) as Id;
            let nota = 1 + g.ate(5) as u8;
            c.registrar_compra(id, produto, Some(nota))?;
        }
    }
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_exemplo_carrega() {
        let c = catalogo_exemplo().unwrap();
        assert_eq!(c.num_categorias(), 6);
        assert_eq!(c.num_produtos(), 18);
        assert_eq!(c.num_clientes(), 8);
    }

    #[test]
    fn gerador_e_deterministico() {
        let mut a = Gerador::new(42);
        let mut b = Gerador::new(42);
        for _ in 0..100 {
            assert_eq!(a.proximo(), b.proximo());
        }
    }

    #[test]
    fn catalogo_gerado_tem_o_tamanho_pedido() {
        let c = gerar_catalogo(200, 300, 4, 7).unwrap();
        assert_eq!(c.num_clientes(), 200);
        assert_eq!(c.num_produtos(), 300);
        assert_eq!(c.num_categorias(), 3);
    }
}
