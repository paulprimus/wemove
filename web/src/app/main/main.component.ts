import { Component, OnInit, signal } from '@angular/core';
import { MainService } from '../api';
import type { MainResponse } from '../api';

@Component({
  selector: 'app-main',
  imports: [],
  templateUrl: './main.component.html',
  styleUrl: './main.component.css',
})
export class MainComponent implements OnInit {
  protected readonly mainJson = signal<string>('');
  protected readonly error = signal<string>('');

  ngOnInit(): void {
    MainService.mainGet()
      .then((response: MainResponse) => {
        this.mainJson.set(JSON.stringify(response, null, 2));
      })
      .catch((err: unknown) => {
        this.error.set(err instanceof Error ? err.message : 'Unknown error');
      });
  }
}
