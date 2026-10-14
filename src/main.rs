//! Interface de linha de comando do ConectaStore.
//!
//! Uso:
//!   cargo run -- demo                            demonstração completa
//!   cargo run -- produtos                        lista o catálogo
//!   cargo run -- produto <id>                    consulta um produto
//!   cargo run -- buscar <texto>                  busca produtos pelo nome
//!   cargo run -- cadastrar <id> <nome> <categoria> <preço>
//!   cargo run -- recomendar-cliente <id> [limite]
//!   cargo run -- recomendar-produto <id> [limite]
//!   cargo run --release -- medir                 medição de desempenho

use std::process::ExitCode;

use megastore::catalogo::Catalogo;
use megastore::dados::catalogo_exemplo;
use megastore::medicao;
use megastore::modelo::Id;
use megastore::recomendacao::{Recomendacao, recomendar_para_cliente, recomendar_para_produto};

const AJUDA: &str = "\
ConectaStore - recomendação de produtos baseada em grafos

Comandos:
  demo                                   demonstração completa (padrão)
  produtos                               lista o catálogo de exemplo
  produto <id>                           consulta um produto pelo id
  buscar <texto>                         busca produtos pelo nome
  cadastrar <id> <nome> <categoria> <preço>
                                         cadastra um produto e mostra a consulta
  recomendar-cliente <id> [limite]       recomenda para um cliente
  recomendar-produto <id> [limite]       quem comprou este produto também comprou
  medir                                  medição de desempenho (use --release)
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match executar(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("erro: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn executar(args: &[String]) -> Result<(), String> {
    let comando = args.first().map(String::as_str).unwrap_or("demo");

    if comando == "medir" {
        medicao::executar(&medicao::VOLUMES).map_err(|e| e.to_string())?;
        return Ok(());
    }
    if comando == "ajuda" || comando == "--help" || comando == "-h" {
        print!("{AJUDA}");
        return Ok(());
    }

    let mut catalogo = catalogo_exemplo().map_err(|e| e.to_string())?;

    match comando {
        "demo" => demo(&catalogo),
        "produtos" => {
            listar_produtos(&catalogo);
            Ok(())
        }
        "produto" => {
            let id = ler_id(args.get(1), "id do produto")?;
            mostrar_produto(&catalogo, id)
        }
        "buscar" => {
            let texto = args.get(1).ok_or("informe o texto da busca")?;
            let achados = catalogo.buscar_por_nome(texto);
            if achados.is_empty() {
                println!("Nenhum produto com \"{texto}\" no nome.");
            }
            for p in achados {
                println!("[{}] {} - R$ {:.2}", p.id, p.nome, p.preco);
            }
            Ok(())
        }
        "cadastrar" => {
            let id = ler_id(args.get(1), "id do produto")?;
            let nome = args.get(2).ok_or("informe o nome")?;
            let categoria = ler_id(args.get(3), "id da categoria")?;
            let preco: f64 = args
                .get(4)
                .ok_or("informe o preço")?
                .parse()
                .map_err(|_| "preço inválido".to_string())?;
            catalogo
                .cadastrar_produto(id, nome, categoria, preco)
                .map_err(|e| e.to_string())?;
            println!("Produto cadastrado.");
            mostrar_produto(&catalogo, id)
        }
        "recomendar-cliente" => {
            let id = ler_id(args.get(1), "id do cliente")?;
            let limite = ler_limite(args.get(2))?;
            let recs = recomendar_para_cliente(&catalogo, id, limite).map_err(|e| e.to_string())?;
            let nome = catalogo
                .buscar_cliente(id)
                .map(|c| c.nome.as_str())
                .unwrap_or("?");
            println!("Recomendações para {nome} (cliente {id}):");
            mostrar_recomendacoes(&catalogo, &recs);
            Ok(())
        }
        "recomendar-produto" => {
            let id = ler_id(args.get(1), "id do produto")?;
            let limite = ler_limite(args.get(2))?;
            let recs = recomendar_para_produto(&catalogo, id, limite).map_err(|e| e.to_string())?;
            let nome = catalogo
                .buscar_produto(id)
                .map(|p| p.nome.as_str())
                .unwrap_or("?");
            println!("Quem comprou \"{nome}\" também comprou:");
            mostrar_recomendacoes(&catalogo, &recs);
            Ok(())
        }
        outro => Err(format!("comando desconhecido: {outro}\n\n{AJUDA}")),
    }
}

fn ler_id(valor: Option<&String>, o_que: &str) -> Result<Id, String> {
    valor
        .ok_or(format!("informe o {o_que}"))?
        .parse()
        .map_err(|_| format!("{o_que} inválido"))
}

fn ler_limite(valor: Option<&String>) -> Result<usize, String> {
    match valor {
        None => Ok(5),
        Some(v) => v.parse().map_err(|_| "limite inválido".to_string()),
    }
}

fn listar_produtos(catalogo: &Catalogo) {
    for p in catalogo.listar_produtos() {
        let categoria = catalogo
            .buscar_categoria(p.categoria_id)
            .map(|c| c.nome.as_str())
            .unwrap_or("?");
        println!(
            "[{:>2}] {:<28} {:<17} R$ {:>8.2}",
            p.id, p.nome, categoria, p.preco
        );
    }
}

fn mostrar_produto(catalogo: &Catalogo, id: Id) -> Result<(), String> {
    let p = catalogo
        .buscar_produto(id)
        .ok_or(format!("produto {id} não encontrado"))?;
    let categoria = catalogo
        .buscar_categoria(p.categoria_id)
        .map(|c| c.nome.as_str())
        .unwrap_or("?");
    println!(
        "Produto {}: {} | categoria: {} | R$ {:.2}",
        p.id, p.nome, categoria, p.preco
    );
    Ok(())
}

fn mostrar_recomendacoes(catalogo: &Catalogo, recs: &[Recomendacao]) {
    if recs.is_empty() {
        println!("  (nenhuma recomendação)");
    }
    for (i, r) in recs.iter().enumerate() {
        let nome = catalogo
            .buscar_produto(r.produto_id)
            .map(|p| p.nome.as_str())
            .unwrap_or("?");
        println!("  {}. {:<28} pontuação {:.3}", i + 1, nome, r.pontuacao);
    }
}

fn demo(catalogo: &Catalogo) -> Result<(), String> {
    let g = catalogo.grafo();
    println!("=== ConectaStore: demonstração ===\n");
    println!(
        "Catálogo: {} produtos, {} clientes, {} categorias",
        catalogo.num_produtos(),
        catalogo.num_clientes(),
        catalogo.num_categorias()
    );
    println!(
        "Grafo: {} vértices, {} arestas\n",
        g.num_vertices(),
        g.num_arestas()
    );

    println!("--- Consulta por id ---");
    mostrar_produto(catalogo, 12)?;

    for cliente in [1, 4] {
        let nome = catalogo
            .buscar_cliente(cliente)
            .map(|c| c.nome.as_str())
            .unwrap_or("?");
        let mut comprados: Vec<&str> = catalogo
            .historico(cliente)
            .into_iter()
            .filter_map(|id| catalogo.buscar_produto(id).map(|p| p.nome.as_str()))
            .collect();
        comprados.sort_unstable();
        println!("\n--- {nome} já comprou: {} ---", comprados.join(", "));
        let recs = recomendar_para_cliente(catalogo, cliente, 5).map_err(|e| e.to_string())?;
        println!("Recomendações para {nome}:");
        mostrar_recomendacoes(catalogo, &recs);
    }

    println!("\n--- Quem comprou \"Mouse sem fio\" também comprou ---");
    let recs = recomendar_para_produto(catalogo, 11, 5).map_err(|e| e.to_string())?;
    mostrar_recomendacoes(catalogo, &recs);
    Ok(())
}
