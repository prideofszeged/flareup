import { invoke } from '@tauri-apps/api/core';

/** Defines how script output should be displayed to the user */
export type ScriptMode = 'fullOutput' | 'compact' | 'silent' | 'inline';

/** Represents a single argument definition for a script command */
export type ScriptArgument = {
	name: string;
	placeholder?: string;
	optional: boolean;
	percentEncoded: boolean;
};

/** Represents a complete script command with metadata and configuration */
export type ScriptCommand = {
	path: string;
	filename: string;
	title: string;
	mode: ScriptMode;
	schemaVersion: number;
	packageName?: string;
	icon?: string;
	authors?: string;
	description?: string;
	arguments: ScriptArgument[];
	needsConfirmation: boolean;
	refreshTime?: string;
};

/** Store managing script commands including loading and execution */
class ScriptCommandsStore {
	commands = $state<ScriptCommand[]>([]);

	constructor() {
		this.loadCommands();
	}

	/** Loads all available script commands from the backend */
	async loadCommands() {
		try {
			this.commands = await invoke('get_script_commands');
		} catch (error) {
			console.error('Failed to load script commands:', error);
		}
	}

	/** Executes a script command with the provided arguments */
	async runCommand(command: ScriptCommand, args: string[]) {
		try {
			const result = await invoke<string>('run_script_command', {
				commandPath: command.path,
				args
			});
			return result;
		} catch (error) {
			console.error('Failed to run script command:', error);
			throw error;
		}
	}

	/** Opens the scripts folder in the system file manager */
	async openScriptsFolder() {
		await invoke('open_scripts_folder');
	}
}

/** Global instance of the script commands store */
export const scriptCommandsStore = new ScriptCommandsStore();
