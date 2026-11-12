# Fauste Player — Guia do utilizador

<div class="ai-notice" role="note">Este guia foi traduzido com IA a partir do guia em inglês (v0.14.0-142-g86653d97, commit 86653d9) e pode conter erros. Em caso de diferença, prevalece a versão em inglês. <a href="../index.html" hreflang="en">Ler o original em inglês</a></div>

O Fauste Player é uma aplicação de emissão (playout) para estúdios de rádio.
Executa vários **leitores** independentes, lado a lado. Cada leitor tem as
suas próprias listas e o seu próprio transporte. Os leitores podem encadear
faixas com misturas sobrepostas, pré-escutar numa saída **CUE** separada e
sobreviver a falhas sem pôr nada no ar por si sós.

> **Criado com ajuda de IA, fornecido tal como está.** O Fauste Player, este guia
> e as suas traduções foram desenvolvidos com ajuda substancial de
> ferramentas de IA. É fornecido sem qualquer garantia e a sua utilização é
> por sua conta e risco: ver o
> [Aviso legal](https://github.com/sanchezfauste/fauste-player#disclaimer).

![Ecrã principal](../../images/main-screen.png)

| Guia | O que abrange |
|---|---|
| [Primeiros passos](getting-started.md) | Instalação, primeiro arranque, adicionar música, reproduzir |
| [Leitores](players.md) | A coluna do leitor: transporte, modos, contagem decrescente, forma de onda, CUE |
| [Listas](playlists.md) | Separadores, a tabela de faixas, arrastar e largar, o menu de contexto, ficheiros M3U/PLS |
| [Cartucheira](cartwall.md) | Cartuchos instantâneos: disparo, páginas, ciclos, cartuchos exclusivos, configuração |
| [Biblioteca](library.md) | A janela da biblioteca de faixas: categorias, tipos, pesquisa, pré-escuta, o editor externo e os ficheiros alterados |
| [Disposição](layout.md) | Mostrar e ocultar painéis, larguras dos leitores, mover e redimensionar a cartucheira, maximizar um painel, o menu da disposição |
| [Automatização](automation.md) | A coluna Grelha, o seu editor, o musicalizador e as suas rotações |
| [Marcadores e mistura](markers-and-mixing.md) | Cue-in/cue-out, o ponto MIX, intro e outro, análise automática |
| [Definições](settings.md) | Saídas de áudio, leitores, análise, listas, MIDI, controlo remoto |
| [Normalização do loudness](loudness.md) | Todas as faixas e cartuchos com o mesmo loudness graças a um ganho fixo por faixa: o indicador de ganho, excluir e ajustar uma faixa, saídas bit-perfect |
| [Saída bit-perfect](bit-perfect.md) | Um modo especial: sem crossfades, volume a 100 %. Dispositivos exclusivos que seguem a frequência de cada ficheiro, o indicador BP, saída DSD, verificar uma cadeia |
| [Teclado](keyboard.md) | Atalhos |
| [Superfícies de controlo MIDI](midi.md) | Botões, faders e luzes em controladores MIDI |
| [Controlo remoto](remote-control.md) | Operar o Fauste Player pela rede: API HTTP, segurança |
| [Dados e cópias de segurança](data-and-backups.md) | Onde ficam os ficheiros, gravação automática, recuperação após uma falha, modo portátil |
| [Android](android.md) | A aplicação em telemóveis e tablets: instalação, a disposição tátil, a música no dispositivo |
| [Resolução de problemas](troubleshooting.md) | Sem som, dispositivo perdido, ficheiros marcados como indisponíveis, registos |

A interface está disponível em inglês, espanhol, catalão, neerlandês,
francês, galego, alemão, italiano, basco, polaco e português. Segue o idioma
do sistema operativo, a menos que escolha outro em Definições. O inglês e o
espanhol incluídos, foi produzida com IA e pode conter erros. As correções
são bem-vindas como issues ou pull requests.

O guia também está disponível noutros idiomas, traduzido com IA a
partir do guia em inglês: no [guia em linha](https://sanchezfauste.com/fauste-player/guide/),
escolha um com o botão do globo na barra superior.
Uma página traduzida indica-o no topo e liga à página em inglês, que é a de
referência quando as duas não coincidem.
