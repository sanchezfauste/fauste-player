# Reprodutores

Cada columna é un reprodutor. Os reprodutores son independentes: cada un ten
as súas propias lapelas de listas, o seu transporte, o seu volume e as súas
saídas.

![O reprodutor 1 en antena: cabeceira coa ganancia de sonoridade, portada, título, pista seguinte, transporte, conta atrás, vúmetro, fader e forma de onda](../../images/guide/player.png)

## Cabeceira {#header}

| Elemento | Significado |
|---|---|
| `P1` … `Pn` | Número do reprodutor (a tecla numérica que o reproduce) |
| Botóns **Ocultar** e **Maximizar** / **Restaurar** | Os dous botóns pequenos do extremo dereito da cabeceira: ocultan o reprodutor, ou maximízano durante un momento (**Restaurar** mentres está maximizado). Aparecen atenuados mentres a disposición está bloqueada, **Ocultar** no último panel visible e **Maximizar** cun só panel visible (ver [Disposición](layout.md#maximize-a-panel)) |
| Punto de estado e etiqueta | **En antena** (vermello), **En pausa** (ámbar), **Detido** (gris) |
| Columnas estreitas | Cando a columna é demasiado estreita para todos os distintivos, a cabeceira vai quitando primeiro os menos importantes (o texto do estado, despois **Outras silenciadas**, o aviso de entrada, **Stop ao final**, **Mestura** / **Fundido**, o punto de estado), así que nada se solapa nunca. O número do reprodutor, **CUE**, o modo, **BP**, o distintivo de ganancia e os botóns **Ocultar** e **Maximizar** quedan sempre. Ensancha a columna para velos todos |
| Distintivo **Mestura** / **Fundido** | Está en curso un fundido cruzado coa pista seguinte, ou un stop con fundido. **Mestura** nunca se acende nun reprodutor cuxa saída Main é bit-perfect: non fai fundidos cruzados (ver [O que custa](bit-perfect.md#what-it-costs)) |
| Distintivo **Stop ao final** | O reprodutor detense cando remata a pista actual |
| Distintivo **Repetir** / **Parar despois** | A pista actual repítese, ou detén o reprodutor cando remata, pola súa propia marca no menú da lista. Pon o punteiro enriba para ver a frase completa. Gaña o botón **Stop ao final** do propio reprodutor: mentres está activado, só se ve o seu distintivo |
| Distintivo de ganancia | Coa [normalización de sonoridade](loudness.md#the-gain-in-the-player-header) activada, a ganancia coa que se reproduce a pista en antena (**+4.0 dB**), ou un **0.0 dB** ámbar cando a pista se reproduce sen cambios (aínda sen medición, unha saída bit-perfect, ou unha pista excluída da normalización). Pon o punteiro enriba para ver a razón. Móstrase mentres o reprodutor reproduce ou está en pausa |
| **BP** / **DSD** | O distintivo só aparece nun reprodutor cuxo dispositivo Main funciona en bit-perfect; **BP** acéndese mentres a pista actual chega sen cambios ao seu dispositivo Main. **DSD** substitúeo mentres unha pista DSD sae como DSD, sen cambios (consulta [Saída bit-perfect](bit-perfect.md)). **Outras silenciadas** aparece ao seu carón cando ese fluxo DSD mantén as outras fontes fóra da saída |
| **SINGLE** \| **CONT** | Modo de reprodución (ver abaixo): un só control unido, a metade acesa é o modo activo |
| **CUE** | Preescoitar a pista seguinte na saída CUE |

Os dous botóns pequenos do extremo dereito da cabeceira, **Ocultar** e
**Maximizar** (ou **Restaurar**), ocultan o reprodutor ou maximízano durante
un momento; os mesmos dous están no menú do clic dereito sobre o fondo da
cabeceira (non sobre un dos seus botóns) (ver
[Disposición](layout.md#maximize-a-panel)).

## Fila de información {#info-row}

- **Portada** da pista en antena, ou unha imaxe de vinilo no seu lugar.
- **Título e artista** da pista en antena. Un reprodutor detido mostra a
  pista que iniciará Play (a súa seguinte), coa súa portada, a súa duración e
  a súa forma de onda, listo no cue-in, ou onde fixeses clic na súa forma de
  onda.
- Unha pista reproducida ou cargada antes de que remate a súa análise xa ten
  a súa duración cando a cabeceira do ficheiro a indica: a conta atrás,
  `transcorrido / total` e o clic para saltar funcionan ao instante, sobre
  unha liña plana ata que a forma de onda estea lista. Cando a cabeceira non
  a garda (ficheiros AAC en bruto, ficheiros MP3 sen cadro de duración,
  Matroska e WebM), o total le «—» e non se pode facer clic na forma de onda
  ata que remate a análise.
- **Vúmetro estéreo** (na columna á dereita do reprodutor, ao carón do fader;
  ocupa a fila de información e o transporte): o nivel que o reprodutor
  entrega, despois do seu volume.
  - Unha barra continua por canle, na escala da norma do tipo de vúmetro (o
    medidor de pico dixital de forma predeterminada: −60 a 0 dBFS, e os 20 dB
    superiores ocupan a metade da altura). A escala é unha regra a cada lado
    das barras, rotulada nas unidades propias do medidor en ambos os lados.
    O seu extremo superior e o inferior (no medidor dixital, o chan da escala)
    están sempre marcados; cada rótulo ten unha marca en cada regra, e as
    marcas máis curtas entre os rótulos funcionan como as dunha regra de
    medir: equidistantes en valores redondos, cada 1 dB no medidor EBU e cada
    5 dB por debaixo de −20 no dixital cando o medidor é o bastante alto, e
    cada 2, 2,5, 5 ou 10 dB (ou ningunha) onde é demasiado baixo para elas. No
    medidor dixital (e no personalizado), un medidor alto rotula máis valores:
    cada 1 dB de −20 a 0, e cada 5 dB entre as marcas de 10 dB por debaixo de
    −20 (−45, −55), sempre equidistantes e só onde caben sen que os rótulos se
    toquen; os outros tipos de medidor conservan os rótulos que dá a súa
    norma. O nivel de aliñamento (−18 dBFS no medidor dixital) é unha marca
    branca máis grosa en ambas as regras. Non se debuxa nada sobre as barras
    nin entre elas, así que o que ves nas barras é só o nivel, a retención do
    pico e as cores.
  - A barra é verde, amarela desde o nivel de aviso (−9 dBFS) e vermella
    desde o nivel de perigo (−3 dBFS). Os outros tipos de medidor póñense
    vermellos onde o fai a súa escala (desde 0 VU, desde o máximo permitido
    nun PPM).
  - Os medidores K-System mostran dúas seccións: a barra sólida é o nivel
    medio (RMS) e a parte máis escura por riba chega ao pico. As súas cores
    son as do K-System: verde por debaixo de 0, ámbar de 0 a +4, vermello por
    riba.
  - Os medidores de pico dixital, K-System e personalizado manteñen o nivel
    máis alto aceso un momento como unha liña (a retención do pico; a súa
    duración é **Retención do pico** en [Configuración → Vúmetros](settings.md#meters),
    e 0 desactívaa). Os medidores PPM EBU, PPM DIN e VU non teñen retención.
  - O número de arriba é o nivel máis alto desde que comezou a entrada, en
    dBFS, en vermello na zona de perigo. Queda despois dun stop e volve
    comezar cando se reproduce unha entrada (a seguinte ou a mesma outra vez),
    ou cando fas clic nel.
  - O número de abaixo é a sonoridade en LUFS (EBU R128), en verde a ±1 LU do
    obxectivo (−23 LUFS).
  - O tipo de medidor e cada nivel pódense cambiar en
    [Configuración → Vúmetros](settings.md#meters).
  - **Lecturas por riba de 0 dBFS.** O medidor mostra o que entrega o
    reprodutor, e iso pode superar a escala completa. A barra detense no
    extremo superior da escala, así que o mesmo vermello mostra 0 dBFS e
    calquera valor por riba; só o número de arriba di canto, co seu signo
    (por exemplo `+3.5`).
    - Un ficheiro pode levar por si mesmo niveis por riba da escala completa
      (un ficheiro de coma flotante, ou un ficheiro con perdas cuxos picos
      decodificados a superan).
    - Converter a frecuencia pode crear picos entre as mostras: un sinal que
      toca 0 dBFS le uns +3 dBFS tras pasar de 44,1 a 48 kHz. A opción de true
      peak le tamén eses picos.
    - O medidor le cada reprodutor por separado, non a suma no dispositivo:
      dous reprodutores nunha mesma saída poden sumarse por riba da escala
      completa sen que ningún dos medidores o mostre.
    - Nada no reprodutor engade ganancia por riba do 100 %. Un dispositivo de
      enteiros satura na escala completa; un dispositivo de coma flotante
      recibe o nivel tal cal e o sistema de son ou o controlador satúrao.
- **Fader de volume** (á dereita do medidor, tan alto coma el). Segue unha
  lei de mesa de mesturas: é fino preto do extremo superior e máis pronunciado
  cara abaixo, así que baixalo un pouco baixa un pouco o nivel (o 90 % do
  percorrido son uns −2,5 dB, o 50 % uns −15,9 dB, o 10 % uns −48 dB). O
  extremo superior é 0 dB e o inferior é silencio. As marcas a través do fader
  sinalan 0, −2, −4, −6, −8, −10, −12, −15, −20, −25, −30, −40, −50 e −∞ dB,
  co nivel escrito á súa dereita.
  - Arrástrao: o fader móvese tanto como o punteiro, desde onde está, así que
    agarralo nunca o fai saltar; un clic sen mover non cambia nada. Tirar
    máis alá do extremo superior ou inferior e volver deixa o pomo no extremo
    ata que o punteiro volve ao punto onde o alcanzou, así que o pomo nunca
    queda lonxe do punteiro.
  - Usa a roda do rato: un paso por cada clic (uns 0,6 dB preto de 0 dB).
  - Mantén **Ctrl** ou **Shift** mentres arrastras ou usas a roda para movelo
    dez veces máis fino.
  - Fai dobre clic no pomo para volver a 0 dB (un dobre clic noutro punto do
    fader non fai nada).

  O nivel en dB (cun decimal, ou −∞ para o silencio) está escrito baixo o
  fader, na mesma fila que a liña de sonoridade do medidor, e a dica tamén o
  mostra. Unha sesión gardada antes desta lei conserva o nivel de cada
  reprodutor; só cambia a posición do fader.
  Nun reprodutor cuxa saída Main é bit-perfect o fader queda bloqueado ao
  100 % e en gris, le un 0.0 atenuado, e a súa dica explica a razón (ver [O que custa](bit-perfect.md#what-it-costs)).
- **Título, artista** e a liña **seguinte**, cun cadrado verde. Mentres o CUE
  está activo, a posición de preescoita móstrase en azul.

## Transporte {#transport}

| Botón | Acción |
|---|---|
| **Play / Seguinte** (grande) | Detido: inicia a pista seguinte. En antena: pasa con fundido á pista seguinte (o tempo de fundido axústase en [Configuración](settings.md)); mentres dura o fundido dun fundido cruzado iniciado por Seguinte ou por un segue, unha banda verde baleira o botón durante a duración do fundido, co bordo movéndose de esquerda a dereita coma o dun cartucho (un fundido cruzado iniciado por Anterior baleira o botón Anterior). En pausa: retoma. Se a pista en antena é tamén a seguinte, Play reiníciaa co fundido habitual. Nunha saída bit-perfect cambia ao instante, sen fundido, e a súa dica dío (ver [O que custa](bit-perfect.md#what-it-costs)). |
| **Stop** | Detén ao instante (cunha rampla curta contra os chasquidos) |
| **Stop con fundido** | Fundido de saída e stop. Mentres dura un stop con fundido, unha banda baleira o botón durante a duración do fundido, co bordo movéndose de esquerda a dereita |
| **Pausa** | Pausa ou retoma; pestanexa en ámbar mentres está en pausa |
| **Stop ao final** (un triángulo de play e despois un cadrado) | Detén cando remata a pista actual, unha vez. No modo SINGLE só está dispoñible mentres a pista actual se repite: pon fin á repetición cando remata a pasada que está soando. Para parar despois dunha pista cada vez que se reproduce, ou para repetir unha pista, usa o seu menú na lista (consulta [Listas](playlists.md)) |
| **Anterior** (unha barra e dous triángulos) | En antena: volve con fundido á pista que este reprodutor reproduciu antes, como fai Seguinte. Prémeo de novo para seguir retrocedendo. A pista que deixaches pasa a ser a seguinte. Nunha saída bit-perfect cambia ao instante, sen fundido (ver [O que custa](bit-perfect.md#what-it-costs)). |
| **Reiniciar** (unha barra e un triángulo) | Volve ao comezo da pista actual (o seu cue-in). Un reprodutor en pausa segue en pausa. |

Desde o teclado, `Up` e `Down` moven a selección pola lista, `Enter` fixa
a pista seleccionada como seguinte, e un segundo `Enter` sobre ela dentro do
tempo dun dobre clic reprodúcea. Consulta [Teclado](keyboard.md).

Os botóns que agora non poden actuar aparecen atenuados: Stop e Reiniciar
sen nada cargado, Pausa e Stop con fundido mentres está detido, Anterior sen
pista anterior ou durante un fundido. Un reprodutor lembra as últimas 50
pistas que reproduciu (`players.history_len` en `config.json`, de 0 a 1000).

## Modos {#modes}

- **CONT (continuo):** no punto MIX o reprodutor inicia a pista seguinte e
  superpón o final da actual. Consulta
  [Marcadores e mestura](markers-and-mixing.md).
- **SINGLE:** cada pista detense ao seu final. *Stop ao final* non está
  dispoñible neste modo, porque cada pista xa se detén, salvo mentres a pista
  actual se repite: entón pon fin á repetición cando remata a pasada que está
  soando.

## Conta atrás {#countdown}

O número grande é o tempo que queda ata o final da pista (o seu cue-out), con
décimas. O tempo transcorrido e o total están na fila baixo a forma de onda,
á dereita. Durante os últimos segundos antes do final (10 de forma
predeterminada, axustable en Configuración) a conta atrás pestanexa en
vermello.

## Forma de onda {#waveform}

- A parte xa reproducida debúxase coa cor da forma de onda; o resto é máis
  escuro. Cunha cor de forma de onda violeta, a liña MIX violeta destaca
  menos; escolle outra cor de forma de onda se precisas distinguilas.
- O contorno mostra os picos, tenue; o corpo sólido dentro del é o nivel medio
  (RMS). Nunha pista forte os picos ocupan toda a altura, e o corpo segue
  mostrando onde a pista é máis suave ou máis forte.
- Unha área azul sombreada ao comezo marca a **intro**, e un distintivo conta
  atrás nela. A intro só se mostra cando se estableceu.
- Unha área laranxa sombreada ao final marca o **outro**, coa súa propia conta
  atrás.
- Unha liña violeta descontinua (a cor do distintivo **Mestura**, para non
  confundila co outro laranxa) cunha etiqueta **MIX** marca onde empeza a
  pista seguinte no modo continuo. Aparece atenuada no modo single, e nun
  reprodutor cuxa saída Main é bit-perfect, que non mestura (ver
  [O que custa](bit-perfect.md#what-it-costs)).
- Debúxase o ficheiro enteiro. O comezo e o final en silencio que a reprodución
  salta (antes do cue-in e despois do cue-out) debúxanse máis escuros, cunha
  liña fina onde a reprodución empeza e remata. Con **Usar cue-in e cue-out**
  desactivado (Configuración → Reprodutores) nada é máis escuro e as dúas liñas
  aparecen atenuadas: a reprodución vai desde o comezo ata o final do ficheiro,
  e o cue-in e o cue-out desta guía significan eses dous extremos.
- Pon o punteiro enriba para ver o tempo que hai debaixo del. **Fai clic para
  saltar** alí. Nun reprodutor detido, un clic escolle onde **Play** inicia a
  pista seguinte: a cabeza de reprodución e a conta atrás móvense alí, e non
  soa nada ata que premes Play. Escoller outra pista seguinte, movela ou
  quitala, ou Stop, volve ao cue-in; tamén o fai calquera outra forma de
  iniciar unha pista, e Reiniciar, Anterior e o avance automático usan sempre
  o cue-in. Un clic antes do cue-in (no comezo máis escuro) escolle o cue-in.
  Un clic no cue-out ou despois del (na cola máis escura) cancela unha
  elección anterior: Play inicia no cue-in. Un clic é premer e soltar sen mover
  o punteiro máis duns poucos píxeles.
- **Premer e arrastrar** move a vista ampliada ao longo da pista, como se a
  agarrases. Un arrastre nunca salta, e sen zoom non fai nada. Alt-arrastrar
  segue editando marcadores.
- **Roda do rato** sobre a forma de onda: amplía e reduce arredor do punteiro,
  ata o maior detalle que ten a análise. **Shift+roda** (ou unha roda lateral)
  móvese ao longo da pista. Co zoom activo, a vista segue a posición de
  reprodución, salvo durante 10 segundos despois de ampliala ou movela
  (`ui.follow_current_grace_secs`; 0 desactiva o seguimento). **Vista
  completa**, na esquina superior dereita, reducir o zoom ao máximo ou unha
  pista nova volven mostrar a pista enteira.

## CUE (preescoita) {#cue-pre-listen}

Premer **CUE** (ou **Preescoitar no CUE** no menú dunha pista) reproduce a
pista na saída CUE do reprodutor, por exemplo uns auriculares, e abre unha
pequena **xanela CUE** para ese reprodutor. Pode
haber varias xanelas abertas, unha por reprodutor. Consulta
[Configuración](settings.md) para escoller o dispositivo CUE. Un reprodutor
necesita unha saída Cue: sen ela, **CUE** e
**Preescoitar no CUE** aparecen atenuados, e ao poñer o punteiro enriba dilo.

Cunha soa tarxeta de son, a saída Cue pode ser a propia saída de antena (ou
compartila con outro reprodutor ou coa cartucheira). Unha preescoita aí
óese en antena, así que:

- mentres nada soa nesa saída, o CUE comeza ao instante;
- mentres algo soa alí (un reprodutor en reprodución ou en pausa, un
  cartucho, outra preescoita, a preescoita da biblioteca), o CUE pregunta
  primeiro: **Preescoitar igualmente** ou **Cancelar** (Cancelar é a opción
  predeterminada: Esc e Enter cancelan). A mesma pregunta vén do menú da
  pista, da xanela CUE, da cartucheira e da xanela da biblioteca;
- cando algo comeza nesa saída mentres soa unha preescoita que non
  confirmaches, a preescoita para e a barra de estado di por que. Unha
  preescoita que confirmaches segue soando.

A dica do botón CUE di cando a súa preescoita sae pola saída de antena.

![A xanela CUE do reprodutor 4, preescoitando a súa pista seguinte](../../images/guide/cue-window.png)

A xanela mostra:

- o título e o artista;
- a forma de onda de todo o ficheiro coa posición do CUE. Funciona como a do
  reprodutor: fai clic para saltar alí, amplía coa roda, arrastra para moverte,
  **Vista completa**, as contas atrás da intro e do outro, e os marcadores de
  intro, outro e MIX, que editas aquí como no reprodutor (consulta
  [Marcadores e mestura](markers-and-mixing.md)). Un CUE reproduce o ficheiro
  enteiro, así que nada se debuxa máis escuro, o cue-in e o cue-out son liñas
  atenuadas, e o outro conta atrás ata o final do ficheiro. O seu zoom é
  propio: a forma de onda do reprodutor non se move. Mentres o CUE soa, unha
  vista ampliada segue a súa posición como a do reprodutor; un CUE en pausa
  conserva a vista que fixaches, para que poidas ampliar e colocar
  marcadores. Un CUE iniciado despois de pechar a súa xanela, ou noutra pista,
  mostra o ficheiro enteiro;
- o tempo transcorrido e o tempo que queda ata o final do ficheiro (un CUE
  reproduce ficheiros enteiros);
- **Pausar o CUE** / **Retomar o CUE**, **Parar o CUE** e **Marcar como
  seguinte**. **Marcar como seguinte** fai da pista en CUE a seguinte do
  reprodutor e mantén o CUE soando. Aparece atenuado cando a pista xa é a
  seguinte. Se a pista en CUE é a que está en antena, reprodúcese unha vez
  máis cando remate a pasada actual.

Mentres o CUE está en pausa, o seu botón **Pausar o CUE** (que se mostra como
**Retomar o CUE**) pestanexa en ámbar, como o propio do reprodutor.

Un salto nun CUE en pausa mantéñeo en pausa. O botón de pechar da xanela, ou
**Parar o CUE**, detén o CUE.

Mentres un CUE está en marcha, marcar unha pista como seguinte (dobre clic)
ou facer un só clic nunha fila move o CUE a esa pista, desde o seu cue-in; se
estaba en pausa, volve reproducir. Unha pista cuxo ficheiro falta ou é
ilexible deixa o CUE onde está.
