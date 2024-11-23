import '@mantine/core/styles.css';
import { Button } from './lib';
import { ThemeProvider } from './components/ThemeProvider';

function App() {
  return (
    <>
      <ThemeProvider>
        <Button>some button</Button>
      </ThemeProvider>
    </>
  );
}

export default App;
