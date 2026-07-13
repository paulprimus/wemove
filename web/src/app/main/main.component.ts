import { Component, OnInit, signal } from '@angular/core';
import { HelloService } from '../api';
import type { HelloWorldResponse } from '../api';

@Component({
  selector: 'app-main',
  imports: [],
  templateUrl: './main.component.html',
  styleUrl: './main.component.css',
})
export class MainComponent implements OnInit {
  protected readonly helloWorldJson = signal<string>('');
  protected readonly error = signal<string>('');

  ngOnInit(): void {
    HelloService.helloWorld()
      .then((response: HelloWorldResponse) => {
        this.helloWorldJson.set(JSON.stringify(response, null, 2));
      })
      .catch((err: unknown) => {
        this.error.set(err instanceof Error ? err.message : 'Unknown error');
      });
  }
}
