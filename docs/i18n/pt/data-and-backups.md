# Dados e cópias de segurança

## Onde ficam os ficheiros {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Definições (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Listas e sessão | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache de análise | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Registos e relatórios de falhas | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Os caminhos do Linux seguem as variáveis XDG (`XDG_CONFIG_HOME` e outras)
quando estão definidas.

## Modo portátil {#portable-mode}

Defina a variável de ambiente `FAUSTE_HOME` para uma pasta e tudo passa a
ficar guardado aí, em `config/`, `data/`, `cache/` e `logs/`. É útil numa
pen USB, ou para manter configurações separadas lado a lado.

## Ficheiros {#files}

| Ficheiro | Conteúdo |
|---|---|
| `config.json` | Definições, saídas, limiares de análise, afinação avançada |
| `playlists.json` | Listas, faixas, marcas de tocada, marcadores manuais |
| `session.json` | Para cada leitor: lista visível, faixas atual e seguinte, modo, posição, volume, larguras das colunas (por coluna) |

## Gravação automática e cópias de segurança {#autosave-and-backups}

- As alterações são guardadas cerca de um segundo depois de acontecerem.
  Enquanto algo toca, a sessão (posições) é atualizada ao mesmo ritmo.
- Cada gravação escreve um ficheiro temporário e só depois substitui o
  antigo, por isso um corte de energia nunca deixa um ficheiro escrito a
  meio.
- As versões anteriores são mantidas como `.bak1`, `.bak2` e `.bak3`.
- Se um ficheiro não puder ser lido, é usada a cópia de segurança válida mais
  recente. O ficheiro ilegível é mantido, com o nome alterado para
  `*.corrupt-<hora>`, para inspeção. A aplicação arranca sempre.

## Recuperação após uma falha {#crash-recovery}

Após uma falha ou um reinício, cada leitor regressa com a sua lista, as
faixas atual e seguinte e a sua posição, mas **em pausa ou parado**. Nada
vai para o ar por si só. Um leitor que estava mesmo no fim da sua faixa
regressa ao início dessa faixa, para que premir Play a toque em vez de a
terminar de imediato.

## Editar `config.json` à mão {#editing-configjson-by-hand}

Feche primeiro a aplicação (se houver áudio no ar, pergunta antes de fechar).
Os valores desconhecidos ou fora do intervalo são corrigidos para o valor
válido mais próximo quando o ficheiro é carregado, e as correções ficam
registadas. As secções `limits` e `tuning` contêm valores avançados (limites
de recursos, temporização do motor) que não estão na janela de Definições.
