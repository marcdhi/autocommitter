import React, { useState } from 'react';
import { dialog } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api';
import './App.css';

function App() {
  const [rustProjectRoot, setRustProjectRoot] = useState('');
  const [sourcePath, setSourcePath] = useState('');
  const [destPath, setDestPath] = useState('');
  const [repoUrl, setRepoUrl] = useState('');
  const [command, setCommand] = useState('');
  const [output, setOutput] = useState('');

  const handleFolderSelect = async (setter) => {
    try {
      const selected = await dialog.open({
        directory: true,
        multiple: false,
        title: 'Select Folder'
      });
      if (selected) {
        setter(selected);
      }
    } catch (err) {
      console.error("Failed to open folder selector:", err);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      const cmd = `cd "${rustProjectRoot}" && ./target/release/ac "${sourcePath}" "${destPath}" "${repoUrl}"`;
      setCommand(cmd);

      // Execute the command
      const result = await invoke('execute_command', { command: cmd });
      setOutput(result);
      console.log(result);
    } catch (error) {
      console.error('Error generating or executing command:', error);
      setOutput(`Error: ${error.toString()}`);
    }
  };

  return (
    <div className="App">
      <h1>AutoCommitter Configuration</h1>
      
      <form onSubmit={handleSubmit}>
        <div>
          <label htmlFor="rustProjectRoot">Rust Project Root:</label>
          <input
            type="text"
            id="rustProjectRoot"
            value={rustProjectRoot}
            onChange={(e) => setRustProjectRoot(e.target.value)}
            placeholder="Select Rust project root"
            required
          />
          <button type="button" onClick={() => handleFolderSelect(setRustProjectRoot)}>
            Browse
          </button>
        </div>

        <div>
          <label htmlFor="sourcePath">Source Repository Path:</label>
          <input
            type="text"
            id="sourcePath"
            value={sourcePath}
            onChange={(e) => setSourcePath(e.target.value)}
            placeholder="Select source path"
            required
          />
          <button type="button" onClick={() => handleFolderSelect(setSourcePath)}>
            Browse
          </button>
        </div>

        <div>
          <label htmlFor="destPath">Destination Repository Path:</label>
          <input
            type="text"
            id="destPath"
            value={destPath}
            onChange={(e) => setDestPath(e.target.value)}
            placeholder="Select destination path"
            required
          />
          <button type="button" onClick={() => handleFolderSelect(setDestPath)}>
            Browse
          </button>
        </div>

        <div>
          <label htmlFor="repoUrl">Remote Repository URL:</label>
          <input
            type="text"
            id="repoUrl"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            placeholder="e.g., https://github.com/marcdhi/compare_test.git"
            required
          />
        </div>

        <button type="submit">Generate and Execute Command</button>
      </form>

      {command && (
        <div>
          <h2>Generated Command:</h2>
          <pre>{command}</pre>
        </div>
      )}

      {output && (
        <div>
          <h2>Command Output:</h2>
          <pre>{output}</pre>
        </div>
      )}
    </div>
  );
}

export default App;
