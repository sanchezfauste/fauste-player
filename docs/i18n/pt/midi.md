# Superfícies de controlo MIDI

O Fauste Player pode ser tocado a partir de controladores MIDI:
controladores de pads e faders, superfícies ao estilo DJ ou teclados. Os
botões de transporte e o fader de volume de cada leitor podem ser associados
a um botão, uma tecla ou um fader, e os botões com luzes mostram o que cada
leitor está a fazer.

## Ativar {#turning-it-on}

![Definições, MIDI: o interruptor para ativar o MIDI e a lista de ações, cada uma com um botão Aprender](../../images/guide/settings-midi.png)

Abra **Definições → MIDI** e assinale **Usar superfícies de controlo MIDI**.
A lista em **Portas de entrada** mostra todas as entradas MIDI que o
computador tem e se estão ligadas. Só são abertos os controladores que
associou (alguns sistemas dão uma porta a um só programa de cada vez), mais
todas as entradas enquanto está a aprender um controlo; o programa nunca
escuta as suas próprias portas. Os controladores podem ser ligados e
desligados com o programa em execução: de dois em dois segundos, as portas
são procuradas de novo, e um controlador que regressa é ligado pelo seu nome,
com as suas luzes repostas.

No Linux, o MIDI passa pelo ALSA: o seu utilizador tem de ter permissão para
abrir o sequenciador (`/dev/snd/seq`, normalmente por pertencer ao grupo
`audio`).

## Associar um controlo {#binding-a-control}

Para cada leitor há uma linha por ação: **Play / Seguinte**, **Pausa**,
**Stop**, **Stop com fade**, **Reiniciar**, **Anterior**, **CUE** e
**Volume**.

1. Clique em **Aprender** na linha.
2. Prima o botão ou mova o fader que quer (entretanto diz **Mova um
   controlo…**). Um botão aceita uma tecla, um pad ou um botão que envia uma
   alteração de controlo; o **Volume** aceita um fader ou botão rotativo (uma
   alteração de controlo) ou um fader de pitch bend.
3. A linha mostra o dispositivo e o controlo, por exemplo
   `APC mini · Nota 36, canal 1`.

Se esse controlo já estava associado a outra ação, passa para esta. `Esc` ou
clicar de novo no botão cancela a aprendizagem. **Limpar** remove uma
associação.

## Como se comportam os controlos {#how-the-controls-behave}

- Um botão atua quando é premido (uma tecla ou pad a descer, ou uma alteração
  de controlo a subir pelo meio do seu intervalo), exatamente como o botão do
  leitor no ecrã. Um botão que está esbatido no ecrã não faz nada.
- Um fader move o volume na mesma escala que o fader no ecrã. Para evitar
  saltos, só assume o controlo quando alcança ou ultrapassa o volume atual
  (soft takeover); se o volume for alterado no ecrã, o fader tem de o
  apanhar de novo. Nada muda até mover um controlo: arrancar o programa nunca
  envia nada para o ar.
- Com **Acender os botões (LED)** ativado, os botões associados acendem-se:
  o Play enquanto o leitor está no ar, a Pausa a piscar enquanto está em
  pausa, o CUE durante a pré-escuta, e o Stop, o Stop com fade, o Reiniciar e
  o Anterior enquanto podem atuar. As luzes vão para a porta de saída do
  controlador com o mesmo nome; pode definir-se outra em `config.json`
  (`midi.devices`).
