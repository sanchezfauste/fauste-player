# Saída bit-perfect

Um dispositivo **bit-perfect** recebe as amostras de cada ficheiro
exatamente como estão no ficheiro: a mesma frequência de amostragem, os
mesmos valores, sem reamostragem, alteração de volume nem mistura. É útil
para cadeias de monitorização e ligações digitais, onde qualquer
processamento no computador deve ser evitado.

## Tornar um dispositivo bit-perfect {#setting-a-device-bit-perfect}

1. Em **Definições → Saídas de áudio**, escolha o dispositivo explicitamente
   para a saída Main de um leitor (ou da cartucheira). Um leitor deixado no
   predefinido do sistema não pode ser tornado bit-perfect. Um dispositivo que
   nenhuma saída usa já perde o seu interruptor bit-perfect e o seu modo DSD
   no próximo arranque da aplicação.
2. Escolha **Avançado** no topo da secção. Em **Definições por
   dispositivo**, ative **Bit-perfect** junto ao dispositivo.
3. Reinicie a aplicação.

O dispositivo passa então a arrancar à sua própria frequência de amostragem,
se lhe deu uma no mesmo sítio, e senão à frequência de amostragem global, e
segue cada ficheiro a partir daí.

O interruptor fica desativado quando o dispositivo não pode dar acesso
exclusivo.
- **Linux:** escolha um dispositivo ALSA cujo nome começa por `hw:`. É a
  própria placa de som. O PulseAudio, o PipeWire, o JACK e os dispositivos
  ALSA `default` ou `plughw:` misturam ou convertem, por isso nunca são
  bit-perfect.
- **Windows:** escolha o dispositivo no sistema **WASAPI**. É aberto em modo
  exclusivo.
  - Nas definições de som do Windows, as propriedades **Avançadas** do
    dispositivo têm de ter *Permitir que as aplicações assumam o controlo
    exclusivo deste dispositivo* ativado (vem ativado por predefinição).
  - Enquanto toca, nenhum outro programa pode usar o dispositivo.
- **macOS:** escolha o dispositivo no **Core Audio**. É aberto em hog mode.
  - A frequência de amostragem do dispositivo é definida para a da faixa, e o
    seu formato para o formato inteiro mais largo que oferece a essa
    frequência (as definições que a Configuração de MIDI e Áudio mostra).
  - São devolvidas quando a aplicação deixa de usar o dispositivo.
  - Dois dispositivos com exatamente o mesmo nome não podem ser tornados
    bit-perfect.

## O que acontece num dispositivo bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Acesso exclusivo.** Nada mais no computador pode tocar no dispositivo
  enquanto a aplicação o usa. Se o acesso exclusivo for recusado, o
  dispositivo toca na mesma, partilhado, e o indicador BP fica apagado.
- **A frequência segue o ficheiro.** Quando nada toca no dispositivo e começa
  uma faixa a outra frequência de amostragem, o dispositivo é aberto de novo
  a essa frequência.
  - Isto acontece quando toca uma faixa, retoma uma carregada em pausa,
    pré-escuta ou dispara um cartucho. As faixas que apenas estão à espera (a
    faixa seguinte de cada leitor) são preparadas de novo à nova frequência.
  - A reabertura demora o que o dispositivo precisar para arrancar
    (normalmente algumas dezenas de milissegundos). O arranque fica atrasado
    por esse tempo.
  - Enquanto algo toca no dispositivo, a frequência nunca muda. Uma faixa a
    outra frequência que comece então (por exemplo, uma faixa a 48 kHz
    misturada a seguir a uma a 44,1 kHz, ou uma faixa iniciada enquanto outro
    leitor ou um cartucho toca no mesmo dispositivo) é convertida em toda a
    sua duração e não é bit-perfect.
  - Se o dispositivo recusar uma frequência, mantém a anterior e a faixa é
    convertida.
- **Sem processamento, quando nada o pede.** As amostras passam sem
  alterações enquanto se verificarem todas estas condições:
  - o volume do leitor está a 100 %;
  - nenhum fade está a decorrer;
  - nada mais toca nas mesmas saídas (outro leitor, um cartucho, um tom de
    teste).

## DSD {#dsd}

Um ficheiro DSD normalmente toca convertido para PCM, como qualquer outro
ficheiro. Um dispositivo bit-perfect pode, em vez disso, receber o fluxo DSD
sem alterações.

**Os três modos.** Na vista Avançado de Definições → Saídas de áudio, cada
dispositivo que uma saída usa tem uma escolha **DSD** por baixo do seu
interruptor bit-perfect:
- **Converter para PCM** (a predefinição): o DSD é convertido, como em
  qualquer outro dispositivo.
- **DoP** (DSD over PCM): os bits DSD viajam dentro de amostras PCM de 24
  bits, que a maioria dos conversores com DSD reconhece. Funciona em todos os
  sistemas.
- **DSD nativo** (só Linux): DSD em bruto, para dispositivos ALSA `hw:` cujo
  controlador indica um formato de amostra DSD.

Só são oferecidos os modos que o dispositivo pode aceitar, e uma linha por
baixo da escolha diz porque não são os outros: o dispositivo não está ligado,
o bit-perfect está desativado, o dispositivo não pode ser aberto em
exclusivo, o DSD nativo precisa de Linux, ou o dispositivo não aceita DSD
nativo. Um modo guardado para um dispositivo que não o pode aceitar aparece
agora como PCM, que é o que toca; o modo guardado regressa quando o
dispositivo o puder aceitar de novo. Alterar um modo, a definição de mistura
ou o silêncio DSD precisa de um reinício, como as outras definições de
saída.

**Quando o DSD sai sem alterações.** Têm de se verificar todas estas
condições quando a faixa começa:
- o dispositivo é bit-perfect, com acesso exclusivo, e o seu modo é DoP ou
  DSD nativo;
- a faixa é DSD (DSF ou DFF), mono ou estéreo, e foi analisada (é assim que
  se conhece a sua frequência DSD);
- o volume do leitor está a 100 %;
- nada mais toca no dispositivo (outro leitor, um cartucho, um tom de teste);
- o dispositivo aceita o fluxo. O DoP precisa de uma frequência do
  dispositivo igual à frequência DSD dividida por 16 (176,4 kHz para DSD64,
  352,8 kHz para DSD128, 705,6 kHz para DSD256) e de um formato de 24 ou 32
  bits. O DSD nativo precisa de um dispositivo que aceite o formato DSD a
  essa frequência.

Quando o DSD nativo termina, o dispositivo volta ao PCM à frequência que
tinha antes da faixa DSD, porque muitos conversores aceitam DSD nativo a
frequências que não conseguem tocar como PCM (nenhum conversor toca PCM à
frequência a que corre o DSD512). Uma faixa que continua em PCM mantém a
frequência do fluxo DSD quando o dispositivo a aceita como PCM e, caso
contrário, continua à frequência anterior a partir de onde estava, como tudo
o resto que toca nesse dispositivo; um fade em curso aí termina de imediato.
O dispositivo nunca fica numa frequência que recusa: se nenhuma frequência
abrir (por exemplo, o dispositivo foi desligado nesse momento), a saída fica
perdida até a nova tentativa automática a abrir de novo à frequência
anterior.

Caso contrário, a faixa é convertida para PCM e o registo diz porquê (por
exemplo, «something else plays on the device» ou «the device refused 705600
Hz»). A pré-escuta e os cartuchos são sempre convertidos.

Enquanto o DSD sai sem alterações:
- o indicador do cabeçalho diz **DSD** em vez de **BP**;
- os medidores mostram o nível da conversão PCM da mesma faixa, por isso
  funcionam como de costume;
- o volume tem de ficar a 100 %: a dica do fader di-lo. Movê-lo passa a
  faixa para PCM (ver abaixo);
- o Stop e o Stop com fade param a faixa de imediato, sem fade, uma vez que
  um fluxo DSD não pode ter fade. Premir Play noutra faixa enquanto esta toca
  corta-a da mesma forma em vez de fazer uma mistura;
- a pausa e a retoma também atuam de imediato, sem rampa.

**Silêncio nas pontas.** Cada arranque, fim e passagem para PCM envia
primeiro silêncio DSD (200 ms por predefinição), para que o conversor
sincronize sem estalidos. A exceção é uma faixa DSD que continua um fluxo do
mesmo tipo e frequência DSD cujo silêncio ainda está a decorrer: o conversor
continua sincronizado, por isso começa sem silêncio extra. Uma faixa começa
assim esse tempo mais tarde, e uma passagem para PCM deixa uma pausa dessa
duração. É o **Silêncio DSD** em Definições → Saídas de áudio, Avançado (0 a
2000 ms).

**Quando outra fonte precisa do dispositivo.** **Quando outra fonte precisa
de uma saída DSD**, em Definições → Saídas de áudio, escolhe o que acontece
quando outro leitor, um cartucho ou um tom de teste começa no mesmo
dispositivo (mover o fader do próprio leitor é a exceção: passa sempre a
faixa para PCM):
- **Continuar a faixa DSD em PCM** (a predefinição). O fluxo passa para PCM
  depois do silêncio DSD, e a faixa continua, convertida, a partir de onde
  estava. O mesmo acontece à faixa que se segue por si só (ver abaixo).
- **Manter o DSD e silenciar as outras fontes.** Nada interrompe o fluxo DSD.
  As outras fontes encaminhadas para o dispositivo ficam silenciadas até a
  faixa DSD terminar, e o leitor mostra entretanto um indicador **Outras
  silenciadas**. A faixa seguinte do próprio leitor não se sobrepõe: começa
  quando a faixa DSD termina, sem mistura nem segue. Uma faixa PCM espera
  pelo silêncio DSD; uma faixa DSD do mesmo tipo e frequência DSD continua o
  fluxo sem ele. Mover o fader continua a passar a faixa para PCM.

**Um álbum não fica em DSD com a definição predefinida.** Com *Continuar a
faixa DSD em PCM*, só uma faixa DSD que começa num dispositivo livre sai
como DSD. As faixas que o leitor inicia por si só a seguir (no fim de uma
faixa, num segue ou numa mistura) começam a partir de uma pré-carga, que é
sempre PCM, por isso o dispositivo passa a PCM e elas tocam convertidas. Uma
faixa que inicia você (Play, duplo clique) sai de novo como DSD quando o
dispositivo está livre ou o fluxo DSD anterior ainda está no seu silêncio à
mesma frequência DSD. Para manter um álbum DSD inteiro em DSD, escolha
*Manter o DSD e silenciar as outras fontes*. Então cada faixa do leitor sai
como DSD, e a seguinte começa quando a anterior termina.

Se o dispositivo for perdido enquanto o DSD toca e regressar incapaz de o
transportar (por exemplo, sem acesso exclusivo), a faixa continua em PCM.

## O indicador BP {#the-bp-badge}

O indicador **BP** no cabeçalho do leitor acende-se enquanto a faixa atual
chega ao seu dispositivo Main sem alterações. Têm de se verificar todas
estas condições:

- o dispositivo é bit-perfect e está aberto com acesso exclusivo;
- o dispositivo corre à frequência de amostragem da faixa;
- a faixa é PCM inteiro sem perdas (WAV, AIFF, FLAC, ALAC, WavPack ou
  Monkey's Audio), mono ou estéreo, e de 24 bits no máximo, e o formato do
  dispositivo comporta o tamanho da sua amostra (um ficheiro de 24 bits num
  dispositivo de 16 bits não é bit-perfect). O DSD é convertido, por isso
  nunca acende o BP; quando sai sem alterações (ver [DSD](#dsd)), o
  indicador diz **DSD**;
- a faixa foi analisada, uma vez que é assim que se conhecem a sua frequência
  e o tamanho da sua amostra. As faixas que uma versão anterior analisou
  recebem o seu formato quando analisadas de novo (o aviso de arranque, ou
  Definições → Análise), ou assim que um leitor as mostra ou um cartucho as
  contém;
- o volume está a 100 %, nenhum fade decorre e nada mais toca nas mesmas
  saídas.

Alguns ficheiros nunca são mostrados como bit-perfect:
- **Ficheiros com perdas** (MP3, AAC, Ogg Vorbis, Opus): as suas amostras
  descodificadas não são os valores inteiros que um dispositivo aceita.
- **Ficheiros acima de 24 bits:** o mixer trabalha em vírgula flutuante de
  32 bits, que transporta 24 bits com exatidão.
- **Ficheiros com mais de dois canais:** são misturados para estéreo.

## Verificar por si {#checking-it-yourself}

Para verificar uma cadeia de ponta a ponta:

1. Ligue a saída digital do dispositivo (S/PDIF, AES ou loopback USB) a um
   gravador que capture bit a bit.
2. Toque um ficheiro de teste sem perdas a 100 % sem mais nada a tocar.
3. Grave-o.
4. Compare a gravação com o ficheiro. Por exemplo, com o SoX, inverta um e
   misture-os: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` depois de
   alinhar os seus inícios. Todas as amostras da diferença têm de ser zero.

Os testes automáticos do projeto verificam a mesma propriedade dentro da
aplicação, num dispositivo simulado.

### DSD num conversor real {#dsd-on-a-real-converter}

Os testes automáticos só verificam o DSD em dispositivos simulados. O DoP e o
DSD nativo não foram experimentados pelo projeto num conversor real. Para
verificar um:
1. Defina o dispositivo como **DoP** (ou **DSD nativo** no Linux), reinicie e
   toque um ficheiro DSD a 100 % sem mais nada a tocar. O cabeçalho tem de
   mostrar **DSD**, e o próprio visor do conversor deve mostrar a frequência
   DSD (por exemplo, DSD64) em vez de uma frequência PCM. Um conversor que
   mostra uma frequência PCM ou toca ruído não reconhece o fluxo: volte a
   **Converter para PCM**.
2. Ouça se há um estalido ou uma rajada de ruído no arranque, no Stop, no fim
   da faixa e ao mover o fader. Um estalido significa que o conversor precisa
   de um **Silêncio DSD** mais longo (Definições → Saídas de áudio,
   Avançado).
3. Inicie um cartucho ou outro leitor no mesmo dispositivo, uma vez com cada
   definição de mistura, e verifique o comportamento descrito acima.
4. No Linux, para verificar o DSD nativo sem a aplicação, execute
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Abre o dispositivo em DSD nativo a DSD64 e toca um segundo de silêncio
   DSD. Tem de passar, e o conversor deve sincronizar em DSD64.
5. Com ficheiros DSD em `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   toca-os através do motor num dispositivo simulado e compara as palavras
   com os bytes do ficheiro (ver [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
