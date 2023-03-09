import { useEffect, useRef, useState } from "react";
import { ToastContainer, toast } from "react-toastify";
import React from "react";
import Spreadsheet, { Matrix, Point } from "react-spreadsheet";

import "react-toastify/dist/ReactToastify.css";
import "./App.css";

const GRID_SIZE = 90;
const GRID_CELL_SIZE = 10;
const GRID_PIXEL = GRID_CELL_SIZE * GRID_SIZE;

const devmode = process.env.NODE_ENV === "development";
const server_address = devmode
  ? "localhost:8080"
  : "bplace-api.preview.blackfoot.dev";
const secure = !devmode;

function setValue(value: string, position: {x: number,y: number}, canvas: HTMLCanvasElement) {
  // TODO
}

function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [username, setUsername] = useState<string | null | undefined>(
    localStorage.getItem("username")
  );
  const [currentWs, setCurrentWs] = useState<WebSocket>();
  const [username_input, setUsername_input] = useState<string>();
  const [data, setData] = useState([
    [{ value: "Vanilla" }, { value: "Chocolate" }],
    [{ value: "Strawberry" }, { value: "Cookies" }],
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
      if (canvasRef.current) {
        let query = await fetch(
          `http${secure ? "s" : ""}://${server_address}/`
        );
        let canvas_pixels = await query.json();
        for (const pixel of canvas_pixels) {
          // todo sync values
        }
      }
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
      }  else if (canvasRef.current) setValue(message, {x:1,y:1}, canvasRef.current);
    };
  }, [username, canvasRef.current]);



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
      <Spreadsheet data={data} onSelect={(selected: Point[]) => { console.log(selected) }} onChange={(data: any) => { 
        console.log(data );
        setData(data)
      }} />
    </div>
  );
}

export default App;
