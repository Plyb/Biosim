import { listen } from '@tauri-apps/api/event';
import './App.scss';

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api';

type UpdateWorldPayload = {
  cells: ('Dead' | 'Alive')[][]
}

function App() {
  let [gridItems, setGridItems] = useState([[false]]);

  useEffect(() => {
    invoke('get_world_width').then(worldWidth => {
      const WORLD_WIDTH = worldWidth as any as number;
      console.log(WORLD_WIDTH);
      setGridItems(
        Array.from({ length: WORLD_WIDTH })
          .map(() => Array.from({ length: WORLD_WIDTH })
            .map(() => false)));
    });
  }, []);
  
  useEffect(() => {
    listen('update-world', ({payload} : {payload: UpdateWorldPayload}) => {
      setGridItems(payload.cells.map(row => row.map(cell => cell === 'Alive')));
    });
  }, []);
  
  return (
    <div className='App'>
      <table className='grid'>
        {gridItems.map((row, x) =>
          <tr key={'row-' + x}>
            {row.map((cell, y) => <td className={cell ? 'filled' : 'empty'} key={`cell-${x},${y}`}></td>)}
          </tr>)}
      </table>
    </div>
  );
}

export default App;
