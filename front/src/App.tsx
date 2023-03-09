import { useEffect, useRef, useState } from "react";
import { ToastContainer, toast } from "react-toastify";
import React from "react";
import Spreadsheet, { Matrix, Point } from "react-spreadsheet";

import "react-toastify/dist/ReactToastify.css";
import "./App.css";

const GRID_SIZE = 90;

const devmode = process.env.NODE_ENV === "development";
const server_address = devmode
  ? "localhost:8080"
  : "bplace-api.preview.blackfoot.dev";
const secure = !devmode;

function setValue(message: {value: string, position: {row: number,column: number}}, gridCells: {value:string}[][], setGridCells: any) {
  const newGrid = JSON.parse(JSON.stringify(gridCells));
  newGrid[message.position.row][message.position.column] = { value: message.value };
  setGridCells(newGrid);
}

function App() {
  const [username, setUsername] = useState<string | null | undefined>(
    localStorage.getItem("username")
  );
  const [currentWs, setCurrentWs] = useState<WebSocket>();
  const [username_input, setUsername_input] = useState<string>();
  const [selected, setSelected] = useState<Point>();
  const [gridCells, setGridCells] = useState<{value:string}[][]>([
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
    [{value: ""},{value: ""},{value: ""},{value: ""},{value: ""}],
  ]);

  useEffect(() => {
    if (!username) return;
    let ws: WebSocket;
    if (!currentWs) {
      ws = new WebSocket(
        `ws${secure ? "s" : ""}://${server_address}/ws/${username}`
      );
      setCurrentWs(ws);
    } else {
      ws = currentWs;
    }
    ws.onopen = async () => {
      console.log("ws opened");
      localStorage.setItem("username", username);
        let query = await fetch(
          `http${secure ? "s" : ""}://${server_address}/`
        );
        let grid_values = await query.json();
        let newGrid = gridCells;
        for (const cell of grid_values) {
          console.log(cell);
          newGrid[cell.position.row][cell.position.column] = {value: cell.value};
        }
        console.log(newGrid);
        
        setGridCells(newGrid)
    };
    ws.onclose = (ev: CloseEvent) =>
      console.log(
        toast("WebSocket close, refresh the page", { autoClose: false }),
        ev
      );
    ws.onmessage = (e) => {
      const message = JSON.parse(e.data);
      if (message.error) {
        toast(message.error);
      }  else setValue(message, gridCells, setGridCells);
    };
  }, [username, setGridCells]);



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
      <Spreadsheet data={gridCells} onSelect={(selected: Point[]) => { setSelected(selected[0]) }} onChange={(data: any) => { 
        if(selected) {
          let value = data[selected.row][selected.column]?.value;
          let update =  JSON.stringify({ position: selected, value });
          console.log({update});
          currentWs?.send(update);
        }
      }} />
    </div>
  );
}

export default App;
