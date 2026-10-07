# Marcadores e mistura

Cada faixa tem até cinco **marcadores**, em segundos:

| Marcador | Significado | Como é definido |
|---|---|---|
| Cue-in | Onde a reprodução começa | Automático: pouco antes do primeiro som acima do limiar de corte |
| Cue-out | Onde a faixa termina | Automático: logo depois do último som acima do limiar de corte |
| MIX (início do segue) | Onde a faixa seguinte começa no modo contínuo | Automático (ver abaixo) |
| Início do outro | Onde começa o final da faixa | Automático (ver abaixo) |
| Fim da intro | Fim da introdução falada por cima | À mão, ou a partir de uma etiqueta `INTRO` no ficheiro |

Os marcadores definidos à mão prevalecem sempre: uma nova análise nunca os
substitui.

## Editar marcadores {#editing-markers}

Na forma de onda de um leitor, ou na forma de onda da sua janela de CUE (o
mesmo menu e as mesmas pegas; uma alteração aparece nas duas ao mesmo
tempo):

- **Clique com o botão direito** onde quer um marcador e escolha **Pôr o
  cue-in aqui**, **Pôr o fim da intro aqui**, **Pôr o início do outro aqui**,
  **Pôr o ponto MIX aqui** ou **Pôr o cue-out aqui**. **Repor os marcadores
  automáticos** remove os marcadores que colocou, e a faixa é analisada de
  novo.
- **Mantenha Alt premido** (Option no macOS): aparecem pegas nos marcadores.
  Arraste uma para a mover; o tempo é mostrado enquanto arrasta. Um
  arrastamento nunca move a posição de reprodução.

O cue-in tem de ficar antes do cue-out. Os outros marcadores ficam entre
eles. As alterações à faixa no ar aplicam-se de imediato à sua próxima
transição.

## A etiqueta INTRO {#the-intro-tag}

Um ficheiro pode trazer o tempo da sua intro numa etiqueta `INTRO`, em
segundos (`12.5`) ou `m:ss`. O nome pode estar escrito com qualquer
capitalização (`INTRO`, `Intro`). Pode ser uma frame de texto de utilizador
ID3v2 (MP3, WAV, AIFF, DSF), um comentário Vorbis, Opus ou FLAC, um item APE
(WavPack, Monkey's Audio) ou um átomo livre MP4. É lida durante a análise.
Um fim de intro manual continua a prevalecer.

## Como são encontrados os marcadores automáticos {#how-the-automatic-markers-are-found}

A análise mede os picos da faixa em passos de 10 ms e o seu loudness em
janelas curtas (50 ms por predefinição).

- **Cue-in / cue-out:** só o quase-silêncio no início e no fim é saltado:
  tudo o que tenha um pico que alcance o *limiar de corte* (−60 dBFS por
  predefinição), em qualquer dos canais, é mantido, com uma *margem de corte*
  (20 ms por predefinição) à volta. Fade-ins suaves, caudas baixas e sons
  curtos nunca são cortados.
- **MIX:** a análise encontra o último ponto em que a faixa ainda está menos
  do que a *queda para o segue* (15 dB por predefinição) abaixo do seu
  próprio loudness típico, de modo que masters altos e baixos com o mesmo
  fade se misturem da mesma forma. Esse ponto nunca fica a mais do que a
  *duração máxima da mistura* (4 s por predefinição) antes do cue-out, para
  que as sobreposições sejam curtas.
- **Outro:** a análise percorre a faixa de trás para a frente a partir do
  cue-out e encontra onde o nível desce mais do que a *queda de nível do
  outro* (6 dB por predefinição) abaixo do loudness mediano da faixa. O outro
  nunca é mais longo do que 30 s por predefinição.
- As faixas mais curtas do que a *duração mínima para marcadores de mistura e
  outro* (60 s por predefinição), como jingles e anúncios, não têm MIX nem
  outro.

### Gravações longas {#long-recordings}

Um programa inteiro (uma, quatro ou mais horas) é analisado como uma canção,
enquanto toca se for preciso: um ficheiro FLAC ou Opus de 4 horas demora
menos de um minuto num computador atual, e a memória não cresce com a
duração. Procurar qualquer ponto, mesmo perto do fim, é imediato. A forma de
onda e os marcadores são guardados na cache de análise até cerca de 16 horas
de áudio; um ficheiro mais longo também funciona, mas é analisado de novo de
cada vez que o Fauste Player arranca.

Todos estes valores estão em **Definições → Análise**. Depois de os alterar,
as faixas são analisadas de novo automaticamente.

## O que o leitor faz com eles {#what-the-player-does-with-them}

- **Modo contínuo com mistura automática ativada:** no ponto MIX, a faixa
  seguinte começa com o nível total enquanto a atual desvanece até ao seu
  cue-out. A sobreposição é exata à amostra.
- **Modo contínuo sem ponto MIX,** ou com a mistura automática desativada: a
  faixa seguinte começa exatamente no cue-out, sem interrupção.
- **Modo single**, ou **Stop no fim**: o leitor pára no cue-out.
- **Premir Play com uma faixa no ar:** a faixa seguinte começa de imediato e
  a atual desvanece durante a *duração do fade* (1 s por predefinição).
- **Usar cue-in e cue-out desativado** (Definições → Leitores): cada leitor
  toca cada faixa de 0 até ao fim do ficheiro. O cue-in e o cue-out,
  automáticos e manuais, são mantidos, e a forma de onda desenha-os como
  linhas esbatidas. O ponto MIX, a intro e o outro continuam a funcionar,
  dentro do ficheiro inteiro; a **Mistura automática no ponto MIX** é um
  interruptor independente. As contagens decrescentes, a coluna de duração,
  os totais das listas nas Definições e os tempos da API remota seguem o
  mesmo intervalo. Os cartuchos usam sempre o seu próprio cue-in e cue-out.
  Alterar a definição nunca reinicia, procura nem pára uma faixa que está a
  tocar; a faixa seguinte é preparada de novo.

Uma faixa pode ser tocada antes de a sua análise terminar. Até lá, toca do
início ao fim do ficheiro, sem ponto MIX.
