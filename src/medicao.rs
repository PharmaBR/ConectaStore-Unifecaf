//! Medição de desempenho com volumes crescentes de dados.
//!
//! Para cada volume: gera o catálogo, mede o tempo de construção, o tempo médio
//! de uma recomendação, o tempo médio de consulta por id (HashMap) e o tempo de
//! uma busca por nome (varredura linear), e estima a memória do grafo como lista
//! e como matriz de adjacência.
//!
//! Rode sempre em modo otimizado: `cargo run --release -- medir`.

use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::dados::{Gerador, gerar_catalogo};
use crate::modelo::{ErroCatalogo, Id};
use crate::recomendacao::recomendar_para_cliente;

#[derive(Debug, Clone, Copy)]
pub struct Volume {
    pub clientes: u32,
    pub produtos: u32,
    pub compras_por_cliente: u32,
}

pub const VOLUMES: [Volume; 4] = [
    Volume {
        clientes: 1_000,
        produtos: 500,
        compras_por_cliente: 5,
    },
    Volume {
        clientes: 10_000,
        produtos: 5_000,
        compras_por_cliente: 5,
    },
    Volume {
        clientes: 100_000,
        produtos: 50_000,
        compras_por_cliente: 5,
    },
    Volume {
        clientes: 1_000_000,
        produtos: 500_000,
        compras_por_cliente: 5,
    },
];

#[derive(Debug, Clone)]
pub struct Resultado {
    pub volume: Volume,
    pub vertices: usize,
    pub arestas: usize,
    pub construcao: Duration,
    pub recomendacao_media: Duration,
    pub consulta_id_media: Duration,
    pub busca_nome: Duration,
    pub memoria_lista_bytes: usize,
    pub memoria_matriz_bytes: u128,
}

const AMOSTRA_RECOMENDACOES: u32 = 1_000;
const AMOSTRA_CONSULTAS: u32 = 100_000;

pub fn medir_volume(volume: Volume) -> Result<Resultado, ErroCatalogo> {
    let inicio = Instant::now();
    let catalogo = gerar_catalogo(
        volume.clientes,
        volume.produtos,
        volume.compras_por_cliente,
        2026,
    )?;
    let construcao = inicio.elapsed();

    let mut sorteio = Gerador::new(99);

    let inicio = Instant::now();
    for _ in 0..AMOSTRA_RECOMENDACOES {
        let cliente = sorteio.ate(volume.clientes as u64) as Id;
        black_box(recomendar_para_cliente(&catalogo, cliente, 10)?);
    }
    let recomendacao_media = inicio.elapsed() / AMOSTRA_RECOMENDACOES;

    let inicio = Instant::now();
    for _ in 0..AMOSTRA_CONSULTAS {
        let produto = sorteio.ate(volume.produtos as u64) as Id;
        black_box(catalogo.buscar_produto(produto));
    }
    let consulta_id_media = inicio.elapsed() / AMOSTRA_CONSULTAS;

    let inicio = Instant::now();
    black_box(catalogo.buscar_por_nome("produto 4242"));
    let busca_nome = inicio.elapsed();

    let grafo = catalogo.grafo();
    Ok(Resultado {
        volume,
        vertices: grafo.num_vertices(),
        arestas: grafo.num_arestas(),
        construcao,
        recomendacao_media,
        consulta_id_media,
        busca_nome,
        memoria_lista_bytes: grafo.memoria_estimada_bytes(),
        memoria_matriz_bytes: grafo.memoria_matriz_bytes(),
    })
}

fn milhar(n: usize) -> String {
    let s = n.to_string();
    let mut saida = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            saida.push('.');
        }
        saida.push(c);
    }
    saida
}

fn memoria(bytes: u128) -> String {
    const KB: f64 = 1024.0;
    let b = bytes as f64;
    if b < KB * KB {
        format!("{:.0} KB", b / KB)
    } else if b < KB * KB * KB {
        format!("{:.1} MB", b / (KB * KB))
    } else if b < KB * KB * KB * KB {
        format!("{:.1} GB", b / (KB * KB * KB))
    } else {
        format!("{:.1} TB", b / (KB * KB * KB * KB))
    }
}

/// Executa todos os volumes e imprime uma tabela em Markdown.
pub fn executar(volumes: &[Volume]) -> Result<Vec<Resultado>, ErroCatalogo> {
    if cfg!(debug_assertions) {
        println!("AVISO: compilado em modo debug. Use `cargo run --release -- medir`.\n");
    }
    println!(
        "| Clientes | Produtos | Vértices | Arestas | Construção | Recomendação (média) | Consulta por id (média) | Busca por nome | Memória lista | Memória matriz |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");

    let mut resultados = Vec::new();
    for &volume in volumes {
        let r = medir_volume(volume)?;
        println!(
            "| {} | {} | {} | {} | {:.1} ms | {:.1} µs | {} ns | {:.2} ms | {} | {} |",
            milhar(r.volume.clientes as usize),
            milhar(r.volume.produtos as usize),
            milhar(r.vertices),
            milhar(r.arestas),
            r.construcao.as_secs_f64() * 1e3,
            r.recomendacao_media.as_secs_f64() * 1e6,
            r.consulta_id_media.as_nanos(),
            r.busca_nome.as_secs_f64() * 1e3,
            memoria(r.memoria_lista_bytes as u128),
            memoria(r.memoria_matriz_bytes),
        );
        resultados.push(r);
    }
    Ok(resultados)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medicao_de_volume_pequeno_produz_numeros_coerentes() {
        let r = medir_volume(Volume {
            clientes: 100,
            produtos: 100,
            compras_por_cliente: 3,
        })
        .unwrap();
        assert_eq!(r.vertices, 100 + 100 + 1);
        assert!(r.arestas >= 100); // ao menos as ligações produto–categoria
        assert!(r.memoria_matriz_bytes > r.memoria_lista_bytes as u128);
    }

    #[test]
    fn formata_milhar() {
        assert_eq!(milhar(1_234_567), "1.234.567");
        assert_eq!(milhar(999), "999");
    }
}
