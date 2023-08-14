import { useEffect, useState } from "react";
import { ToastContainer, toast } from "react-toastify";
import React from "react";
import Spreadsheet, { Point } from "react-spreadsheet";

import "react-toastify/dist/ReactToastify.css";
import "./App.css";

const GRID_SIZE = 20;

const devmode = process.env.NODE_ENV === "development";
const server_address = devmode
  ? "192.168.1.128:8080"
  : "bplace-api.preview.blackfoot.dev";
const secure = !devmode;

function setValue(
  message: { value: string; position: { row: number; column: number } },
  gridCells: { value: string }[][],
  setGridCells: any
) {
  const newGrid = JSON.parse(JSON.stringify(gridCells));
  newGrid[message.position.row][message.position.column] = {
    value: message.value,
  };
  setGridCells(newGrid);
}

function App() {
  const [username, setUsername] = useState<string | null | undefined>(
    localStorage.getItem("username")
  );
  const [currentWs, setCurrentWs] = useState<WebSocket>();
  const [username_input, setUsername_input] = useState<string>();
  const [selected, setSelected] = useState<Point>();
  const [gridCells, setGridCells] = useState<{ value: string }[][]>(
    Array.from({ length: GRID_SIZE }).fill(
      Array.from({ length: GRID_SIZE }).fill({ value: "" })
    ) as { value: string }[][]
  );

  useEffect(() => {
    if (!username) return;
    if (!currentWs) {
      return setCurrentWs(
        new WebSocket(
          `ws${secure ? "s" : ""}://${server_address}/ws/${username}`
        )
      );
    }
    const ws = currentWs;

    ws.onopen = async () => {
      console.log("ws opened");
      localStorage.setItem("username", username);
      const query = await fetch(
        `http${secure ? "s" : ""}://${server_address}/`
      );
      const grid_values = await query.json();
      const newGrid = JSON.parse(JSON.stringify(gridCells));
      for (const cell of grid_values) {
        console.log(cell);
        newGrid[cell.position.row][cell.position.column] = {
          value: cell.value,
        };
      }
      console.log(newGrid);
      setGridCells(newGrid);
    };
    ws.onclose = (ev: CloseEvent) =>
      console.log(
        toast("WebSocket close, refresh the page", { autoClose: false }),
        ev
      );
  }, [username, currentWs]);

  useEffect(() => {
    if (!currentWs) {
      return;
    }
    currentWs.onmessage = (e) => {
      const message = JSON.parse(e.data);
      console.log("received message:", message);
      if (message.error) {
        toast(message.error);
      } else setValue(message, gridCells, setGridCells);
    };
  }, [currentWs, gridCells, setGridCells]);

  if (!username) {
    return (
      <div className="centered">
        Please insert your username
        <br />
        <input
          type="text"
          onChange={(event) => {
            setUsername_input(event.currentTarget.value);
          }}
        />
        <button onClick={() => setUsername(username_input)}>Save</button>
      </div>
    );
  }

  return (
    <div className="App">
      <ToastContainer
        position="top-left"
        autoClose={5000}
        hideProgressBar={false}
        newestOnTop
        closeOnClick={false}
        rtl={false}
        pauseOnFocusLoss
        draggable={false}
        pauseOnHover
      />
      <Spreadsheet
        data={gridCells}
        onSelect={(selected: Point[]) => {
          setSelected(selected[0]);
        }}
        onChange={(data: any) => {
          if (selected) {
            const value = data[selected.row][selected.column]?.value;
            const select = JSON.stringify({ Select: { ...selected }});
            currentWs?.send(select);

            const update = JSON.stringify({ NewGridValue: { position: selected, value }});
            console.log({ update });
            currentWs?.send(update);
          }
        }}
      />
    </div>
  );
}

export default App;
