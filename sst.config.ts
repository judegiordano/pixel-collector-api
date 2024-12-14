
/// <reference path='./.sst/platform/config.d.ts' />

const domain = 'judethings.com'

export default $config({
  app(input) {
    return {
      name: 'pixel-collector-api',
      removal: 'remove',
      home: 'aws',
      providers: { aws: { region: 'us-east-1' } },
      stage: input?.stage
    };
  },
  async run() {
    const { stage } = $app;
    const environment = {
      STAGE: stage,
      LOG_LEVEL: process.env.LOG_LEVEL,
      MONGO_URI: process.env.MONGO_URI,
      GOOGLE_CLIENT_ID: process.env.GOOGLE_CLIENT_ID,
      GOOGLE_CLIENT_SECRET: process.env.GOOGLE_CLIENT_SECRET,
      JWT_SECRET: process.env.JWT_SECRET
    }

    const bucket = new sst.aws.Bucket('assets');

    const authTable = new sst.aws.Dynamo('table', {
      transform: { table: { name: 'pixel_collector_users' } },
      fields: { id: 'string', username: 'string' },
      primaryIndex: { hashKey: 'id' },
      globalIndexes: { username_idx: { hashKey: 'username' } }
    })

    const api = new sst.aws.Function('api', {
      runtime: 'provided.al2023',
      handler: 'bootstrap',
      bundle: 'target/lambda/api',
      memory: '500 MB',
      timeout: '10 minutes',
      architecture: "arm64",
      url: { cors: { allowCredentials: true } },
      logging: { retention: '1 week', format: 'json' },
      environment: {
        ...environment,
        BUCKET_NAME: bucket.name
      },
      link: [bucket, authTable]
    });

    const router = new sst.aws.Router('router', {
      invalidation: false,
      routes: { '/*': api.url },
      domain: {
        name: `api.pixel-collector.${domain}`,
        redirects: [`www.api.pixel-collector.${domain}`]
      }
    })

    return {
      api: api.url,
      url: router.url,
      table: authTable.name,
      bucket: bucket.name,
    }
  },
});
