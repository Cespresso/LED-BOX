package com.example.ledboxdebug

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.runtime.collectAsState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LedBoxScreen(viewModel: LedBoxViewModel) {
    val connectionState by viewModel.connectionState.collectAsState()
    val currentMode by viewModel.currentMode.collectAsState()
    val displayData by viewModel.displayData.collectAsState()
    val currentToolsSubmode by viewModel.currentToolsSubmode.collectAsState()
    val logs by viewModel.logs.collectAsState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("LED BOX Debug") },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            )
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            ConnectionSection(
                state = connectionState,
                onScan = viewModel::startScan,
                onDisconnect = viewModel::disconnect,
            )

            if (connectionState == ConnectionState.Ready) {
                ModeSection(
                    currentMode = currentMode,
                    onModeSelect = viewModel::writeMode,
                    onReadMode = viewModel::readMode,
                )

                if (currentMode == 2) {
                    ToolsSubmodeSection(
                        currentSubmode = currentToolsSubmode,
                        onSubmodeSelect = viewModel::writeToolsSubmode,
                        onReadSubmode = viewModel::readToolsSubmode,
                    )
                }

                DisplaySection(
                    data = displayData,
                    onTogglePixel = viewModel::togglePixel,
                    onSend = { viewModel.writeDisplayData(displayData) },
                    onClear = viewModel::clearDisplay,
                    onRead = viewModel::readDisplayData,
                )
            }

            LogSection(logs = logs)
        }
    }
}

@Composable
private fun ConnectionSection(
    state: ConnectionState,
    onScan: () -> Unit,
    onDisconnect: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text("Connection", style = MaterialTheme.typography.titleMedium)

            Text(
                text = "Status: ${state.name}",
                color = when (state) {
                    ConnectionState.Ready -> Color(0xFF4CAF50)
                    ConnectionState.Disconnected -> Color.Gray
                    ConnectionState.Bonding -> Color(0xFFFF5722)
                    else -> Color(0xFFFFA000)
                }
            )

            if (state == ConnectionState.Disconnected) {
                Text(
                    text = "Passkey: 123456",
                    style = MaterialTheme.typography.bodySmall,
                    color = Color.Gray,
                )
            }

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(
                    onClick = onScan,
                    enabled = state == ConnectionState.Disconnected,
                ) {
                    Text("Scan & Connect")
                }
                OutlinedButton(
                    onClick = onDisconnect,
                    enabled = state != ConnectionState.Disconnected,
                ) {
                    Text("Disconnect")
                }
            }
        }
    }
}

@Composable
private fun ModeSection(
    currentMode: Int?,
    onModeSelect: (Int) -> Unit,
    onReadMode: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Mode", style = MaterialTheme.typography.titleMedium)
                OutlinedButton(onClick = onReadMode) {
                    Text("Read", fontSize = 12.sp)
                }
            }

            if (currentMode != null) {
                Text(
                    text = "Current: 0x%02X (%s)".format(
                        currentMode,
                        LedBoxViewModel.MODE_NAMES[currentMode] ?: "unknown"
                    ),
                    fontFamily = FontFamily.Monospace,
                )
            }

            for (row in 0..1) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    for (col in 0..2) {
                        val mode = row * 3 + col
                        val name = LedBoxViewModel.MODE_NAMES[mode] ?: "?"
                        val isActive = currentMode == mode

                        if (isActive) {
                            Button(
                                onClick = { onModeSelect(mode) },
                                modifier = Modifier.weight(1f),
                            ) {
                                Text(name, fontSize = 12.sp)
                            }
                        } else {
                            OutlinedButton(
                                onClick = { onModeSelect(mode) },
                                modifier = Modifier.weight(1f),
                            ) {
                                Text(name, fontSize = 12.sp)
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun DisplaySection(
    data: IntArray,
    onTogglePixel: (Int, Int) -> Unit,
    onSend: () -> Unit,
    onClear: () -> Unit,
    onRead: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Text("Display (8x8)", style = MaterialTheme.typography.titleMedium)

            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                for (row in 0..7) {
                    Row {
                        for (col in 0..7) {
                            val isOn = (data[row] shr (7 - col)) and 1 == 1
                            Box(
                                modifier = Modifier
                                    .size(36.dp)
                                    .padding(1.dp)
                                    .background(if (isOn) Color.Red else Color(0xFFE0E0E0))
                                    .border(1.dp, Color.Gray)
                                    .clickable { onTogglePixel(row, col) }
                            )
                        }
                    }
                }
            }

            Text(
                text = data.joinToString(" ") { "0x%02X".format(it) },
                fontFamily = FontFamily.Monospace,
                fontSize = 11.sp,
            )

            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onSend) { Text("Send") }
                OutlinedButton(onClick = onClear) { Text("Clear") }
                OutlinedButton(onClick = onRead) { Text("Read") }
            }
        }
    }
}

@Composable
private fun ToolsSubmodeSection(
    currentSubmode: Int?,
    onSubmodeSelect: (Int) -> Unit,
    onReadSubmode: () -> Unit,
) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("Tools Sub-Mode", style = MaterialTheme.typography.titleMedium)
                OutlinedButton(onClick = onReadSubmode) {
                    Text("Read", fontSize = 12.sp)
                }
            }

            if (currentSubmode != null) {
                Text(
                    text = "Current: %d (%s)".format(
                        currentSubmode,
                        LedBoxViewModel.TOOLS_SUBMODE_NAMES[currentSubmode] ?: "unknown"
                    ),
                    fontFamily = FontFamily.Monospace,
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                LedBoxViewModel.TOOLS_SUBMODE_NAMES.forEach { (id, name) ->
                    val isActive = currentSubmode == id
                    if (isActive) {
                        Button(
                            onClick = { onSubmodeSelect(id) },
                            modifier = Modifier.weight(1f),
                        ) {
                            Text(name, fontSize = 12.sp)
                        }
                    } else {
                        OutlinedButton(
                            onClick = { onSubmodeSelect(id) },
                            modifier = Modifier.weight(1f),
                        ) {
                            Text(name, fontSize = 12.sp)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun LogSection(logs: List<String>) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text("Logs", style = MaterialTheme.typography.titleMedium)

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 200.dp)
                    .verticalScroll(rememberScrollState()),
            ) {
                logs.asReversed().forEach { entry ->
                    Text(
                        text = entry,
                        fontSize = 10.sp,
                        fontFamily = FontFamily.Monospace,
                        lineHeight = 13.sp,
                    )
                }
            }
        }
    }
}
