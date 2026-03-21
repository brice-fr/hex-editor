// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: 2026 Brice LECOLE

/**
 * api.js — thin abstraction over Tauri's `invoke` for the hex-editor backend.
 *
 * All functions return Promises. Errors from Rust propagate as rejected
 * Promises with a string message.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Read a file from the filesystem.
 * @param {string} path  Absolute path to the file.
 * @returns {Promise<number[]>}  Raw bytes as a JSON array.
 */
export async function openFile(path) {
  return invoke('open_file', { path });
}

/**
 * Parse an Intel HEX payload.
 * @param {number[]} data  Raw bytes (from openFile).
 * @returns {Promise<string>}  JSON string of HexFile structure.
 */
export async function parseIntelHex(data) {
  return invoke('parse_intel_hex', { data });
}

/**
 * Parse a Motorola S-record payload.
 * @param {number[]} data  Raw bytes (from openFile).
 * @returns {Promise<string>}  JSON string of SrecFile structure.
 */
export async function parseSrec(data) {
  return invoke('parse_srec', { data });
}

/**
 * Detect the file format from its path (extension + magic bytes).
 * @param {string} path
 * @returns {Promise<string>}  "ihex" | "srec" | "binary" | "unknown"
 */
export async function detectFileFormat(path) {
  return invoke('detect_file_format', { path });
}

/**
 * Serialise records to the given format and write them to disk.
 * @param {Array<{record_type: string, address: number, data: number[]}>} records
 * @param {string} path    Absolute destination path.
 * @param {string} format  "ihex" or "srec"
 * @returns {Promise<void>}
 */
export async function saveFile(records, path, format) {
  return invoke('save_file', { records, path, format });
}
